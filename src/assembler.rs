use crate::instruction::{Opcode, Operand, COPY_STATIC_HEADER};
use crate::{parse_u8_literal, VecExt};
use anyhow::anyhow;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use yeet_ops::yeet;

/// LEG-Architecture uses fixed-length instructions.
pub const INST_LENGTH: u8 = 4;

#[derive(Debug)]
pub struct Assembler {
    consts: HashMap<String, u8>,
    labels: HashMap<String, u16>,
    sections: Sections,
    /// The `copystatic` portion.
    binary_header: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AssemblyTarget {
    pub commented_binary: String,
    /// Target binary
    pub binary: BinaryParts,
}

#[derive(Debug, Clone)]
pub struct BinaryParts {
    pub header: Vec<u8>,
    pub code: Vec<u8>,
}

impl BinaryParts {
    pub fn merge(&self) -> Vec<u8> {
        self.header
            .iter()
            .chain(self.code.iter())
            .copied()
            .collect()
    }
}

impl Assembler {
    pub fn new<S: AsRef<str>>(code: S) -> anyhow::Result<Self> {
        let code = code.as_ref();

        let mut consts: HashMap<String, u8> = HashMap::new();
        let mut copy_static_info = (0_u8, 0_u8);
        let mut copy_static_data: Option<Vec<u8>> = None;
        let mut binary_header = Vec::new();

        let sections = Sections::new(code)?;

        if let Some(s) = sections.find("consts") {
            for x in &s.body_lines {
                let x = Self::remove_comment(x);
                if x.is_empty() {
                    continue;
                }
                let mut split = x.split(' ');
                let name = split.next().ok_or(anyhow!(".consts: syntax error: {x}"))?;
                let value = split.next().ok_or(anyhow!(".consts: syntax error: {x}"))?;
                consts.insert(
                    name.into(),
                    parse_u8_literal(value).ok_or(anyhow!("Invalid u8 literal: {x}"))?,
                );
            }
        }

        let code_section = sections
            .find("code")
            .ok_or(anyhow!("Missing .code section"))?;
        let mut labels = Self::read_labels(&code_section.body_lines);

        // parse .data section
        if let Some(s) = sections.find("data") {
            let mut static_data = Vec::new();

            let mem_start = s.args.first().ok_or(anyhow!(".data: missing mem_start"))?;
            let mut mem_start =
                parse_u8_literal(mem_start).ok_or(anyhow!("Invalid mem_start: {mem_start}"))?;
            copy_static_info.1 = mem_start;
            for line in &s.body_lines {
                let line = Self::remove_comment(line);
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
            // here mem_start is the static data length
            copy_static_info.0 = mem_start;
            copy_static_data = Some(static_data);
        }

        // correct labels
        // add the initial 4-byte `copystatic` instruction along with its data
        for x in labels.values_mut() {
            *x += INST_LENGTH as u16;
            *x += copy_static_data
                .as_ref()
                .map(|x| x.len())
                .unwrap_or_default() as u16;
        }

        let entry_section = sections
            .find("entry")
            .ok_or(anyhow!("Missing .entry section"))?;
        let entrypoint = entry_section
            .args
            .first()
            .ok_or(anyhow!(".entry: missing entrypoint"))?;

        let &entrypoint_addr = labels
            .get(entrypoint)
            .ok_or(anyhow!("Cannot find entrypoint: {entrypoint}"))?;
        binary_header.push_all(
            [
                COPY_STATIC_HEADER,
                copy_static_info.0,
                copy_static_info.1,
                entrypoint_addr
                    .try_into()
                    .map_err(|_| anyhow!("entrypoint does not support 16-bit address"))?,
            ]
            .into_iter(),
        );
        if let Some(x) = copy_static_data {
            let mut data_string = String::new();
            for &x in &x {
                use fmt::Write;
                write!(&mut data_string, "{} ", x).unwrap();
            }
            if data_string.len() > 0 {
                data_string.remove(data_string.len() - 1);
            }
            binary_header.push_all(x.iter().copied());
        }

        Ok(Self {
            consts,
            labels,
            sections,
            binary_header,
        })
    }

    fn read_labels(code_section_lines: &[String]) -> HashMap<String, u16> {
        let mut map = HashMap::new();
        let mut offset = 0_u16;
        for line in code_section_lines {
            let line = Self::remove_comment(line.trim());
            if line.is_empty() {
                continue;
            }
            if let Some(x) = line.strip_suffix(':') {
                map.insert(x.into(), offset);
                continue;
            }
            offset += INST_LENGTH as u16;
        }
        map
    }

    pub fn assemble(&self) -> AssemblyTarget {
        let mut commented_binary = String::new();

        let mut commented_binary_append = |b: &[u8], comment: &str| {
            use fmt::Write;
            if b.is_empty() {
                writeln!(&mut commented_binary, "# {}", comment).unwrap();
            } else {
                writeln!(
                    &mut commented_binary,
                    "{} # {}",
                    hex_array_literal(b),
                    comment
                )
                .unwrap();
            }
        };

        commented_binary_append(&self.binary_header[0..4], "copystatic");
        commented_binary_append(&self.binary_header[4..], "data");

        let mut code_binary = Vec::new();
        let code_section = self.sections.find("code").unwrap();
        for line in &code_section.body_lines {
            let line = Self::remove_comment(line.trim());
            // skip labels and empty lines
            if line.ends_with(':') || line.is_empty() {
                commented_binary_append(&[], line);
                continue;
            }

            let inst = self.process_asm_statement(line).unwrap();
            inst.iter().for_each(|&x| code_binary.push(x));
            commented_binary_append(&inst, line);
        }

        let binary_parts = BinaryParts {
            header: self.binary_header.clone(),
            code: code_binary,
        };
        AssemblyTarget {
            binary: binary_parts,
            commented_binary,
        }
    }

    fn process_asm_statement(&self, line: &str) -> anyhow::Result<[u8; 4]> {
        let mut inst = [0_u8; 4];

        let split = line.split(' ').collect::<Vec<_>>();
        let &opcode_str = split.first().ok_or(anyhow!("Missing opcode"))?;
        let opcode =
            Opcode::from_str(opcode_str).map_err(|_| anyhow!("Unknown opcode: {}", opcode_str))?;
        inst[0] = opcode as u8;

        // special handles for opcodes that have 16-bit immediate operands
        let operands = if opcode == Opcode::JumpAddrMove || opcode == Opcode::Call {
            // `jamv` and `call` only support label operands for now
            let label = split[1];
            let Some(&label) = self.labels.get(label) else {
                yeet!(anyhow!("Label not found: {label}"))
            };
            let high = (label >> 8) as u8;
            let low = (label & 0x00ff_u16) as u8;
            // LEG uses small-endianness
            vec![Operand::Immediate(low), Operand::Immediate(high)]
        } else {
            split[1..]
                .iter()
                .map(|&x| match x {
                    _ if let Some(&x) = self.consts.get(x) => Ok(Operand::Immediate(x)),
                    _ => Operand::from_str(x).map_err(|_| anyhow!("Cannot parse operand: {x}")),
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        opcode.binary(&operands)
    }

    fn remove_comment(line: &str) -> &str {
        if let Some(x) = line.split_once(';') {
            x.0.trim()
        } else {
            line
        }
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

fn parse_data_array(s: &str) -> Option<Vec<u8>> {
    s.starts_with("[").then_some(())?;
    s.ends_with("]").then_some(())?;
    let content = &s[1..(s.len() - 1)];
    if content.trim().is_empty() {
        return Some(vec![]);
    }
    let mut items = Vec::new();
    for x in content.split(",") {
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

fn hex_array_literal(binary: &[u8]) -> String {
    let mut line = String::new();
    for &x in binary {
        use fmt::Write;
        write!(&mut line, "0x{:02x} ", x).unwrap();
    }
    line.remove(line.len() - 1);
    line
}
