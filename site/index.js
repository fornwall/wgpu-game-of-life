import { resizeCanvasToDisplaySize } from "./resize.js";
import init, { run } from "./generated/wgpu_game_of_life.js";

  init().then(() => {
    setTimeout(() => {
      run();
    }, 0);
    //const canvas = document.getElementById("webgpu-canvas");
    //setTimeout(() => resizeCanvasToDisplaySize(canvas), 0);
  });

