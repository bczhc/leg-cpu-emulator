use crate::opcode::{Opcode, Operand};
use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use yeet_ops::yeet;
use crate::VecExt;

/// LEG-Architecture uses fixed-length instructions.
const INST_LENGTH: u8 = 4;

#[derive(Debug)]
pub struct Assembler {
    code: String,
    lines: Vec<String>,
    consts: HashMap<String, u8>,
    tc_lines: Vec<String>,
    labels: HashMap<String, u8>,
    sections: Sections,
    /// The `copystatic` portion.
    binary_header: Vec<u8>,
}

#[derive(Debug, Clone)]
struct AssemblyTarget {
    /// Assembly code in "Turing Complete" game
    tc_game_asm: String,
    /// Target binary
    binary: Vec<u8>,
}

impl Assembler {
    pub fn new<S: AsRef<str>>(code: S) -> anyhow::Result<Self> {
        let code = code.as_ref();
        
        let mut tc_lines = Vec::new();
        let mut consts: HashMap<String, u8> = HashMap::new();
        let mut copy_static_info = (0_u8, 0_u8);
        let mut copy_static_data: Option<Vec<u8>> = None;
        let mut binary_header = Vec::new();
        
        let sections = Sections::new(code)?;
        
        if let Some(s) = sections.find("consts") {
            for x in &s.body_lines {
                let mut split = x.split(' ');
                let name = split.next().ok_or(anyhow!(".consts: syntax error"))?;
                let value = split.next().ok_or(anyhow!(".consts: syntax error"))?;
                consts.insert(
                    name.into(),
                    parse_u8_literal(value).ok_or(anyhow!("Invalid u8 literal"))?,
                );
            }
        }

        let code_section = sections
            .find("code")
            .ok_or(anyhow!("Missing .code section"))?;
        code_section
            .body_lines
            .clone()
            .into_iter()
            .for_each(|x| tc_lines.push(x));
        let labels = Self::read_labels(&code_section.body_lines);

        if let Some(s) = sections.find("data") {
            let mut static_data = Vec::new();

            let mem_start = s.args.first().ok_or(anyhow!(".data: missing mem_start"))?;
            let mut mem_start = mem_start.parse::<u8>()?;
            copy_static_info.1 = mem_start;
            for line in &s.body_lines {
                let parts = regex!(r#"^(\S+) (.*?) (\S+)$"#)
                    .capture_vec(line)
                    .ok_or(anyhow!(".data: syntax error"))?;
                let parts = &parts[1..];
                if parts.len() != 2 && parts.len() != 3 {
                    yeet!(anyhow!(".data: syntax error"));
                }
                let data_value =
                    parse_data_value(parts[1]).ok_or(anyhow!(".data: parsing value error"))?;
                let data_byte = match data_value {
                    DataValue::String(s) => s,
                    DataValue::Array(s) => s,
                    DataValue::Byte(b) => {
                        vec![b]
                    }
                };
                data_byte.iter().for_each(|&x| static_data.push(x));
                consts.insert(parts[0].into(), mem_start);
                if let Some(&length_name) = parts.get(2)
                    && length_name != "_"
                {
                    consts.insert(length_name.into(), data_byte.len().try_into()?);
                }
                mem_start += u8::try_from(data_byte.len())?;
            }
            copy_static_info.0 = mem_start;
            copy_static_data = Some(static_data);
        }

        let entry_section = sections
            .find("entry")
            .ok_or(anyhow!("Missing .entry section"))?;
        let entrypoint = entry_section
            .args
            .first()
            .ok_or(anyhow!(".entry: missing entrypoint"))?;

        for x in &consts {
            tc_lines.push(format!("const {} {}", x.0, x.1));
        }
        
        tc_lines.push(Default::default());
        tc_lines.push(format!(
            "copystatic {} {} {}",
            copy_static_info.0, copy_static_info.1, entrypoint
        ));

        let &entrypoint_addr = labels.get(entrypoint).ok_or(anyhow!("Cannot find entrypoint: {entrypoint}"))?;
        binary_header.push_all([Opcode::CopyStatic as u8, copy_static_info.0, copy_static_info.1, entrypoint_addr].into_iter());
        if let Some(x) = copy_static_data {
            let mut data_string = String::new();
            for &x in &x {
                use fmt::Write;
                write!(&mut data_string, "{} ", x).unwrap();
            }
            data_string.remove(data_string.len() - 1);
            tc_lines.push(data_string);
            binary_header.push_all(x.iter().copied());
        }

        tc_lines.push(Default::default());


        Ok(Self {
            code: code.into(),
            lines: code.lines().map(Into::into).collect(),
            consts,
            labels,
            tc_lines,
            sections,
            binary_header,
        })
    }

    fn read_labels(code_section_lines: &[String]) -> HashMap<String, u8> {
        let mut map = HashMap::new();
        // add the initial `copystatic`
        let mut offset = INST_LENGTH;
        for line in code_section_lines {
            let line = line.trim();
            if let Some(x) = line.strip_suffix(':') {
                map.insert(x.into(), offset);
                continue;
            }
            offset += INST_LENGTH;
        }
        map
    }

    fn assemble(&self) -> AssemblyTarget {
        let mut binary = Vec::new();
        binary.push_all(self.binary_header.iter().copied());
        let code_section = self.sections.find("code").unwrap();
        for line in &code_section.body_lines {
            let mut line = line.trim();
            // remove comments
            if let Some(x) = line.rsplit_once(';') {
                line = x.0.trim();
            }
            // skip labels and empty lines
            if line.ends_with(':') || line.is_empty() {
                continue;
            }

            let inst = self.process_asm_statement(line).unwrap().1;
            inst.iter().for_each(|&x| binary.push(x));
        }
        
        AssemblyTarget {
            tc_game_asm: Default::default() /* TODO */,
            binary
        }
    }

    fn process_asm_statement(&self, line: &str) -> anyhow::Result<(String, [u8; 4])> {
        let mut inst = [0_u8; 4];

        let split = line.split(' ').collect::<Vec<_>>();
        let &opcode_str = split.get(0).ok_or(anyhow!("Missing opcode"))?;
        let opcode =
            Opcode::from_str(opcode_str).map_err(|_| anyhow!("Unknown opcode: {}", opcode_str))?;
        inst[0] = opcode as u8;

        let mut process_operands =
            |split_to_inst_indices: &[(usize, usize)]| -> anyhow::Result<()> {
                let mut immediate_mask = 0b00000000_u8;
                let mut process_operand =
                    |split_index: usize, inst_index: usize| -> anyhow::Result<()> {
                        let &operand = split
                            .get(split_index)
                            .ok_or(anyhow!("cp: missing operand"))?;
                        let operand = match operand {
                            _ if let Some(&x) = self.consts.get(operand) => {
                                Operand::Immediate(x)
                            }
                            _ if let Some(&x) = self.labels.get(operand) => {
                                Operand::Immediate(x)
                            }
                            _ => {
                                Operand::from_str(operand)?
                            }
                        };

                        inst[inst_index] = operand.to_u8();
                        if operand.is_immediate() && inst_index == 1 {
                            immediate_mask |= 0b10000000;
                        }
                        if operand.is_immediate() && inst_index == 2 {
                            immediate_mask |= 0b01000000;
                        }
                        Ok(())
                    };
                for x in split_to_inst_indices {
                    process_operand(x.0, x.1)?;
                }
                inst[0] = (inst[0] & 0b00111111) | immediate_mask; /* set the imm. mask bits */
                Ok(())
            };

        match opcode {
            Opcode::Copy => process_operands(&[(1, 1), (2, 3)])?,
            Opcode::Add => process_operands(&[(1, 1), (2, 2), (3, 3)])?,
            Opcode::Load => process_operands(&[(1, 1), (2, 2)])?,
            Opcode::JpLt => process_operands(&[(1, 1), (2, 2), (3, 3)])?,
            Opcode::Halt => process_operands(&[])?,
            _ => {}
        };

        Ok((Default::default() /* TODO */, inst))
    }
}

#[derive(Debug, Clone, Default)]
struct Section {
    name: String,
    args: Vec<String>,
    body_lines: Vec<String>,
}

impl Section {
    fn new(title_line: impl AsRef<str>) -> Section {
        let mut split = title_line.as_ref().split(' ');
        let name = split.next().unwrap();
        let args = split.map(Into::into).collect::<Vec<String>>();
        Self {
            name: name.into(),
            args,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
struct Sections {
    sections: Vec<Section>,
}

impl Sections {
    fn new(code: &str) -> anyhow::Result<Self> {
        let mut sections = Vec::new();
        let mut current: Option<Section> = None;
        for line in code.lines() {
            if line.starts_with('.') {
                if let Some(x) = current.take() {
                    sections.push(x);
                }

                current = Some(Section::new(line.strip_prefix('.').unwrap()));
                continue;
            }
            if let Some(ref mut x) = current
                && !line.is_empty()
            {
                x.body_lines.push(line.into());
            }
        }

        if let Some(x) = current.take() {
            sections.push(x);
        }

        let iter = sections.iter().map(|x| &x.name);
        if let Some(d) = find_duplicates(iter) {
            yeet!(anyhow::anyhow!("Duplicated section not allowed: .{}", d));
        }
        Ok(Self { sections })
    }

    fn find(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|x| x.name == name)
    }
}

fn find_duplicates<T>(iter: impl Iterator<Item = T>) -> Option<T>
where
    T: Eq + PartialEq + Hash + Copy,
{
    let mut set = HashSet::new();
    for x in iter {
        if set.contains(&x) {
            return Some(x);
        }
        set.insert(x);
    }
    None
}

fn regex_capture(regex: Regex, haystack: &str) -> Option<Vec<&'_ str>> {
    regex
        .captures(haystack)?
        .iter()
        .map(|x| x.map(|x| x.as_str()))
        .collect()
}

macro regex($x:expr) {
    ::regex::Regex::new($x).unwrap()
}

trait RegexExt {
    fn capture_vec(self, haystack: &str) -> Option<Vec<&str>>;
}

impl RegexExt for Regex {
    fn capture_vec(self, haystack: &str) -> Option<Vec<&str>> {
        regex_capture(self, haystack)
    }
}

fn parse_quoted_string(s: &str) -> Option<String> {
    s.starts_with("'").then_some(())?;
    s.ends_with("'").then_some(())?;

    let content = &s[1..(s.len() - 1)];
    Some(content.replace("''", "'"))
}

fn parse_u8_literal(s: &str) -> Option<u8> {
    if let Some(x) = s.strip_prefix("0x") {
        u8::from_str_radix(x, 16).ok()
    } else if let Some(x) = s.strip_prefix("0b") {
        u8::from_str_radix(x, 2).ok()
    } else {
        s.parse::<u8>().ok()
    }
}

fn parse_data_array(s: &str) -> Option<Vec<u8>> {
    s.starts_with("[").then_some(())?;
    s.ends_with("]").then_some(())?;
    let mut items = Vec::new();
    for x in s[1..(s.len() - 1)].split(",") {
        items.push(parse_u8_literal(x.trim())?);
    }
    Some(items)
}

fn parse_data_value(value: &str) -> Option<DataValue> {
    match value {
        _ if value.starts_with("'") && value.ends_with("'") => {
            Some(DataValue::String(parse_quoted_string(value)?.into()))
        }
        _ if value.starts_with("[") && value.ends_with("]") => {
            Some(DataValue::Array(parse_data_array(value)?))
        }
        _ if value.parse::<u8>().is_ok() => Some(DataValue::Byte(value.parse().unwrap())),
        _ => None,
    }
}

enum DataValue {
    String(Vec<u8>),
    Array(Vec<u8>),
    Byte(u8),
}

pub static MNEMONICS: Lazy<HashMap<&'static str, u8>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for line in include_str!("../mnemonics.txt").lines() {
        let mut split = line.split(' ');
        let name = split.next().unwrap();
        let value = split.next().unwrap();
        assert_eq!(value.len(), 8);
        let value = u8::from_str_radix(value, 2).unwrap();
        map.insert(name, value);
    }
    map
});

#[cfg(test)]
mod test {
    use crate::asm::{Assembler, Sections};

    macro test_asm($name:literal) {
        include_str!(concat!("../tests/data/", $name, ".asm"))
    }

    #[test]
    fn sections() {
        let code = test_asm!("hello_world");
        let sections = Sections::new(code).unwrap();
        println!("{:?}", sections);
    }

    #[test]
    fn assemble() {
        let code = test_asm!("hello_world");
        let assembler = Assembler::new(code).unwrap();
        println!("{:#?}", assembler);
        println!("{:?}", assembler.assemble());
    }
}
