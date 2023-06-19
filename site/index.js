import init, { run, getRules } from "./generated/wgpu_game_of_life.js";

const ruleSelect = document.getElementById('rule');
const canvas = document.getElementById("webgpu-canvas");
const sizeElement = document.getElementById("size");

canvas.focus();

globalThis.setNewState = function (ruleIdx, size, seed) {
  sizeElement.textContent = size + 'x' + size;
  ruleSelect.value = ruleIdx;
}

try {
  await init();

  for (const [ruleIdx, rule] of getRules().entries()) {
    ruleSelect.appendChild(new Option(rule.name(), ruleIdx));
  }

  await run();
} catch (e) {
  canvas.remove();
  console.log('error', e);
  document.getElementById('fallback').style.display = 'flex';
}
