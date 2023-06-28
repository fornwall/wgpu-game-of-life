import init, { run, getRules, setNewRule, setNewSize, setDensity, resetGame, togglePause, setGenerationsPerSecond } from "./generated/wgpu_game_of_life.js";

const ruleSelect = document.getElementById('rule');
const sizeSelect = document.getElementById("size");
const canvas = document.getElementById("webgpu-canvas");
const overlayElement = document.getElementById("overlay");
const densityInput = document.getElementById("density");
const densityDisplay = document.getElementById("density-display");
const pauseButton = document.getElementById("pauseButton");
const generationsPerSecondInput = document.getElementById("generations-per-second");
const generationsPerSecondDisplay = document.getElementById("generations-per-second-display");
const aboutDialog = document.getElementById('about');
const controls = document.getElementById("hideableControls");

canvas.focus();

function downloadImage() {
  const dataUrl = canvas.toDataURL("image/png");
  const a = document.createElement("a");
  a.href = dataUrl;
  a.download = "game-of-life" + document.title.toLowerCase().replace(' ', '-').replace('/', '-') + ".png";
  a.click();
}

globalThis.setNewState = function (ruleIdx, size, seed, density, paused, generationsPerSecond, frame) {
  document.title = ruleSelect.options[ruleIdx].textContent;
  sizeSelect.value = size;
  ruleSelect.value = ruleIdx;
  const queryString = `?rule=${ruleIdx}&size=${size}&seed=${seed}&density=${density}&gps=${generationsPerSecond}&paused=${paused}` + (paused ? `&frame=${frame}` : '');
  window.history.replaceState({}, '', queryString);

  pauseButton.textContent = paused ? 'Play' : 'Pause';

  densityInput.value = density;
  densityDisplay.innerHTML = '&nbsp;0.' + density;

  generationsPerSecondInput.value = generationsPerSecond;
  generationsPerSecondDisplay.innerHTML = '&nbsp;' + generationsPerSecond;
}

globalThis.toggleFullscreen = function () {
  if (document.fullscreenElement) {
    document.exitFullscreen();
  } else {
    document.documentElement.requestFullscreen();
    if (!controls.classList.contains('hidden')) {
      globalThis.toggleControls();
    }
  }
}

document.documentElement.addEventListener("mousemove", () => {
  overlayElement.classList.remove('hidden');
  setHideTimeout();
});
document.documentElement.addEventListener("touchstart", () => {
  overlayElement.classList.remove('hidden');
  setHideTimeout();
});
let currentHideTimeout = null;
function setHideTimeout() {
  if (currentHideTimeout) {
    clearTimeout(currentHideTimeout);
    currentHideTimeout = null;
  }
  if (controls.classList.contains('hidden')) {
    currentHideTimeout = setTimeout(() => {
      if (controls.classList.contains('hidden')) {
        overlayElement.classList.add('hidden');
        canvas.focus();
      }
      currentHideTimeout = null;
    }, 1500);
  }
}

globalThis.toggleControls = function () {
  controls.classList.toggle('hidden');
  if (controls.classList.contains('hidden')) canvas.focus();
  setHideTimeout();
}

try {
  if (!navigator.gpu) throw new Error("No navigator.gpu");
  await init();

  for (const [ruleIdx, rule] of getRules().entries()) {
    ruleSelect.appendChild(new Option(rule.name(), ruleIdx));
  }
  ruleSelect.addEventListener('change', () => { setNewRule(ruleSelect.value); });
  sizeSelect.addEventListener('change', () => { setNewSize(sizeSelect.value); });
  document.getElementById('downloadButton').addEventListener('click', downloadImage);
  document.getElementById('resetButton').addEventListener('click', resetGame);
  document.getElementById('fullscreenButton').addEventListener('click', toggleFullscreen);
  document.getElementById('hideControlsButton').addEventListener('click', toggleControls);
  document.getElementById('about-link').addEventListener('click', (event) => {
    event.preventDefault();
    aboutDialog.showModal();
  });
  pauseButton.addEventListener('click', togglePause);
  densityInput.addEventListener('change', () => {
    setDensity(densityInput.value);
  });
  generationsPerSecondInput.addEventListener('change', () => {
    setGenerationsPerSecond(generationsPerSecondInput.value);
  });

  const urlParams = new URLSearchParams(window.location.search);
  const rule = parseInt(urlParams.get('rule'));
  const size = parseInt(urlParams.get('size'));
  const seed = parseInt(urlParams.get('seed'));
  const density = parseInt(urlParams.get('density'));
  const paused = "true" === urlParams.get('paused');
  const generationsPerSecond = parseInt(urlParams.get("gps"));

  await run(rule, size, seed, density, paused, generationsPerSecond);
} catch (e) {
  console.error('error', e);
  canvas.remove();
  overlayElement.remove();
  document.getElementById('webgpu-not-working').style.display = 'block';
  document.getElementById('close-dialog').remove();
  aboutDialog.show();
}
