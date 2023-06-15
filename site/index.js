import { resizeCanvasToDisplaySize } from "./resize.js";
import init, { run } from "./generated/wgpu_game_of_life.js";

  init().then(() => {
    const canvas = document.getElementById("webgpu-canvas");
    canvas.focus();
    run();
    //setTimeout(() => resizeCanvasToDisplaySize(canvas), 0);
  });

