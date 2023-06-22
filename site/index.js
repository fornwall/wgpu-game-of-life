import init, { run, getRules, setNewRule, resetGame } from "./generated/wgpu_game_of_life.js";

const ruleSelect = document.getElementById('rule');
const canvas = document.getElementById("webgpu-canvas");
const sizeElement = document.getElementById("size");

canvas.focus();

globalThis.setNewState = function (ruleIdx, size, seed) {
  sizeElement.textContent = size + 'x' + size;
  ruleSelect.value = ruleIdx;
  window.location.hash = `rule=${ruleIdx}&size=${size}&seed=${seed}`;
}

globalThis.toggleFullscreen = function () {
  if (document.fullscreenElement) {
    document.exitFullscreen();
  } else {
    document.documentElement.requestFullscreen();
  }
}

try {
  await init();

  for (const [ruleIdx, rule] of getRules().entries()) {
    ruleSelect.appendChild(new Option(rule.name(), ruleIdx));
  }
  ruleSelect.addEventListener('change', () => {
    setNewRule(ruleSelect.value);
  });
  document.getElementById('resetButton').addEventListener('click', () => {
    resetGame();
  });

  let rule = null;
  let seed = null;
  if (window.location.hash) {
    const urlParams = new URLSearchParams(window.location.hash.substring(1));
    rule = parseInt(urlParams.get('rule'));
    seed = parseInt(urlParams.get('seed'));
  }
  await run(rule, seed);
} catch (e) {
  console.error('error', e);
  canvas.remove();
  document.getElementById('overlay').remove();
  document.getElementById('fallback').style.display = 'flex';
}
