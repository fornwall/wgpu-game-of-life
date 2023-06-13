import { resizeCanvasToDisplaySize } from "./resize.js";
import init from "./generated/wgpu_game_of_life.js";
init().then(() => {
  const canvas = document.getElementById("webgpu-canvas");
  setTimeout(() => resizeCanvasToDisplaySize(canvas), 0);
});

