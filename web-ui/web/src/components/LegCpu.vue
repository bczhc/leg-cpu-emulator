<script setup lang="ts">
import {computed, ref} from "vue";
import {to_hex_padded, useWasm} from "../lib.ts";
import {LegEmulationResult} from "wasm-lib";
import exampleHelloWorld from '../assets/leg-cpu-examples/hello-world/code?raw';
import exampleWaterWorld from '../assets/leg-cpu-examples/water-world/code?raw';
import exampleWaterWorldInput from '../assets/leg-cpu-examples/water-world/input?raw';
import examplePrimeNumbers from '../assets/leg-cpu-examples/prime-numbers/code?raw';

let wasm = useWasm();

let code = ref('');
let assemblyHex = ref('');
let programInput = ref('');
let maxCpuCycles = ref('');
let showRamModel = ref(false);

let emulationResult = ref<LegEmulationResult | null>(null);

function runClick() {
  let cyclesLimit: bigint | undefined;
  try {
    if (maxCpuCycles.value !== '') {
      cyclesLimit = BigInt(maxCpuCycles.value);
    }
  } catch (_) {
  }
  try {
    let target = wasm.LegCpu.assemble(code.value);
    assemblyHex.value = target.commented_binary;
    emulationResult.value = wasm.LegCpu.emulate(target.binary, programInput.value, cyclesLimit);
  } catch (e: any) {
    assemblyHex.value = e.toString();
  }
}

let output_binary_hex = computed(() => {
  let r = emulationResult.value as LegEmulationResult | null;
  if (r == null) {
    return null;
  }
  // Uint8Array's map differs from the Array one. It returns Uint8Array still instead of an array of new type. 😡
  return Array.from(r.output).map(x => to_hex_padded(x, 2)).join(' ');
});

type ExampleKeys = 'hello-world' | 'water-world' | 'prime-numbers';

let examplesOptions: ({ key: ExampleKeys, label: string })[] = [
  {key: 'hello-world', label: 'Hello World'},
  {key: 'water-world', label: 'Water World'},
  {key: 'prime-numbers', label: 'Prime Numbers'},
]

function onExamplesSelected(key: ExampleKeys) {
  switch (key) {
    case "hello-world":
      code.value = (exampleHelloWorld as string).trim();
      programInput.value = '';
      break;
    case "water-world":
      code.value = (exampleWaterWorld as string).trim();
      programInput.value = (exampleWaterWorldInput as string).trim();
      break;
    case 'prime-numbers':
      code.value = (examplePrimeNumbers as string).trim();
      programInput.value = '';
  }
}
</script>

<template>
  <div class="outer">
    <div class="top-view">
      <n-input placeholder="Code" type="textarea" class="side-by-side" :resizable="false"
               v-model:value="code"
      />
      <n-input placeholder="Hex" type="textarea" class="side-by-side" :resizable="false"
               :allow-input="() => false"
               v-model:value="assemblyHex"
      />
    </div>
    <div class="middle-view">
      <n-form
          label-placement="left"
          :label-width="120"
      >
        <n-form-item label="Program Input">
          <n-input v-model:value="programInput"/>
        </n-form-item>
        <n-form-item label="Max CPU cycles">
          <n-input v-model:value="maxCpuCycles"/>
        </n-form-item>
        <div class="buttons-div">
          <n-button type="primary" @click="runClick">Run</n-button>
          <n-button type="info" @click="showRamModel = true">Show RAM</n-button>
          <n-dropdown trigger="click" :options="examplesOptions" @select="onExamplesSelected">
            <n-button type="primary" secondary>Examples</n-button>
          </n-dropdown>
        </div>
      </n-form>
    </div>
    <n-divider style="padding: 0; margin: 0"/>
    <div class="bottom-view">
      <div class="output-line-wrapper" v-if="emulationResult">
        <span>Output: {{ emulationResult!!.output_lossy_string }}</span>
        <span>Output (hex): {{ output_binary_hex }} </span>
        <span>CPU cycles: {{ emulationResult!!.cpu_cycles }}</span>
        <span>Interrupted: {{ emulationResult!!.interrupted }}</span>
      </div>
    </div>
  </div>

  <n-modal v-model:show="showRamModel">
    <n-card
        style="width: 60em"
        title="RAM Viewer"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
    >
      <pre>
      {{ emulationResult?.ram_pretty_hex }}
    </pre>
    </n-card>
  </n-modal>
</template>

<style scoped>
.top-view {
  height: 60%;
}

.outer {
  padding: 0.5em;
  height: 100vh;
}

.output-line-wrapper > span {
  display: block;
}

.side-by-side {
  height: 100%;
  width: 50%;
}

.middle-view {
  margin: 0.5em auto;
}

.buttons-div > * {
  margin: auto .25em;
}
</style>
