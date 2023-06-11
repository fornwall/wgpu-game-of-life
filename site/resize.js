export async function resizeCanvasToDisplaySize(canvas) {
  const devicePixelContentBoxSupported =
    await isDevicePixelContentBoxSupported();
  const observerOptions = devicePixelContentBoxSupported
    ? { box: ["device-pixel-content-box"] }
    : {};
  new ResizeObserver((entries) => {
    const newWidth = devicePixelContentBoxSupported
      ? entries[0].devicePixelContentBoxSize[0].inlineSize
      : canvas.clientWidth * window.devicePixelRatio;
    const newHeight = devicePixelContentBoxSupported
      ? entries[0].devicePixelContentBoxSize[0].blockSize
      : canvas.clientHeight * window.devicePixelRatio;
    canvas.width = newWidth;
    canvas.height = newHeight;
  }).observe(canvas, observerOptions);

  canvas.style.width = "100vw";
  canvas.style.height = "100vh";
}

// https://web.dev/device-pixel-content-box/
function isDevicePixelContentBoxSupported() {
  return new Promise((resolve) => {
    const ro = new ResizeObserver((entries) => {
      resolve(entries.every((entry) => "devicePixelContentBoxSize" in entry));
      ro.disconnect();
    });
    ro.observe(document.body, { box: ["device-pixel-content-box"] });
  }).catch(() => false);
}

