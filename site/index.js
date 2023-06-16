import init, { run } from "./generated/wgpu_game_of_life.js";

const canvas = document.getElementById("webgpu-canvas");
canvas.focus();

try {
  await init();
  await run();
} catch (e) {
  canvas.remove();
  console.log('error', e);
  document.getElementById('fallback').style.display = 'flex';
}
