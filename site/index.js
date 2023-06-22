import init, { run, getRules, setNewRule, setDensity, resetGame } from "./generated/wgpu_game_of_life.js";

const ruleSelect = document.getElementById('rule');
const canvas = document.getElementById("webgpu-canvas");
const sizeElement = document.getElementById("size");
const overlayElement = document.getElementById("overlay");
const densityInput = document.getElementById("density");
const densityDisplay = document.getElementById("density-display");

canvas.focus();

globalThis.setNewState = function (ruleIdx, size, seed, density) {
  document.title = ruleSelect.options[ruleIdx].textContent;
  sizeElement.textContent = size + 'x' + size;
  ruleSelect.value = ruleIdx;
  const queryString = `?rule=${ruleIdx}&size=${size}&seed=${seed}&density=${density}`;
  window.history.replaceState({}, '', queryString);
  densityInput.value = density;
  densityDisplay.innerHTML = '&nbsp;0.' + density;
  overlayElement.style.display = 'block';
}

globalThis.toggleFullscreen = function () {
  if (document.fullscreenElement) {
    document.exitFullscreen();
  } else {
    document.documentElement.requestFullscreen();
  }
}

globalThis.toggleControls = function () {
  const controls = document.getElementById("hideableControls");
  controls.classList.toggle('hidden');
}

try {
  await init();

  for (const [ruleIdx, rule] of getRules().entries()) {
    ruleSelect.appendChild(new Option(rule.name(), ruleIdx));
  }
  ruleSelect.addEventListener('change', () => { setNewRule(ruleSelect.value); });
  document.getElementById('resetButton').addEventListener('click', resetGame);
  document.getElementById('fullscreenButton').addEventListener('click', toggleFullscreen);
  document.getElementById('hideControlsButton').addEventListener('click', toggleControls);

  const urlParams = new URLSearchParams(window.location.search);
  const rule = parseInt(urlParams.get('rule'));
  const seed = parseInt(urlParams.get('seed'));
  const density = parseInt(urlParams.get('density'));

  await run(rule, seed, density);

  densityInput.addEventListener('change', () => {
    setDensity(densityInput.value);
  });
} catch (e) {
  console.error('error', e);
  canvas.remove();
  overlayElement.remove();
  document.getElementById('fallback').style.display = 'flex';
}
