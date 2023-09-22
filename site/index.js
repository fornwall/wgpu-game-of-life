import init, {
  run,
  getRules,
  setNewRule,
  setNewSize,
  setDensity,
  resetGame,
  togglePause,
  setGenerationsPerSecond,
} from "./generated/wgpu_game_of_life.js";

const ruleSelect = document.getElementById("rule");
const sizeSelect = document.getElementById("size");
const canvas = document.getElementById("webgpu-canvas");
const overlayElement = document.getElementById("overlay");
const densityInput = document.getElementById("density");
const densityDisplay = document.getElementById("density-display");
const pauseButton = document.getElementById("pauseButton");
const generationsPerSecondInput = document.getElementById(
  "generations-per-second",
);
const generationsPerSecondDisplay = document.getElementById(
  "generations-per-second-display",
);
const aboutDialog = document.getElementById("about");
const controls = document.getElementById("hideableControls");

canvas.focus();

globalThis.downloadImage = function () {
  const dataUrl = canvas.toDataURL("image/png");
  const a = document.createElement("a");
  a.href = dataUrl;
  a.download =
    "game-of-life-" +
    document.title.toLowerCase().replace(/ |_|'|\//g, "-") +
    ".png";
  a.click();
};

globalThis.setNewState = function (
  ruleIdx,
  size,
  seed,
  density,
  paused,
  generationsPerSecond,
  _frame,
) {
  document.title = ruleSelect.options[ruleIdx].textContent;
  sizeSelect.value = size;
  ruleSelect.value = ruleIdx;
  const hash = `#rule=${ruleIdx}&size=${size}&seed=${seed}&density=${density}&gps=${generationsPerSecond}`;
  window.history.replaceState({}, "", hash);

  pauseButton.textContent = paused ? "Play" : "Pause";

  densityInput.value = density;
  densityDisplay.innerHTML = "&nbsp;0." + density;

  generationsPerSecondInput.value = generationsPerSecond;
  generationsPerSecondDisplay.innerHTML = "&nbsp;" + generationsPerSecond;
};

globalThis.toggleFullscreen = function () {
  if (document.fullscreenElement) {
    document.exitFullscreen();
  } else {
    document.documentElement.requestFullscreen();
    if (!controls.classList.contains("hidden")) {
      globalThis.toggleControls();
    }
  }
  canvas.focus();
};

function setOverlayVisibility(visible) {
  if (visible) {
    overlayElement.classList.remove("hidden");
    canvas.classList.remove("nocursor");
    setHideTimeout();
  } else {
    overlayElement.classList.add("hidden");
    canvas.classList.add("nocursor");
    canvas.focus();
  }
}

for (const event_name of ["mousemove", "touchstart"]) {
  document.documentElement.addEventListener(
    event_name,
    () => {
      setOverlayVisibility(true);
    },
    { passive: true },
  );
}

let currentHideTimeout = null;

function setHideTimeout() {
  if (currentHideTimeout) {
    clearTimeout(currentHideTimeout);
    currentHideTimeout = null;
  }
  if (controls.classList.contains("hidden")) {
    currentHideTimeout = setTimeout(() => {
      if (controls.classList.contains("hidden")) {
        setOverlayVisibility(false);
      }
      currentHideTimeout = null;
    }, 1500);
  }
}

globalThis.toggleControls = function () {
  controls.classList.toggle("hidden");
  setOverlayVisibility(true);
};

try {
  if (!navigator.gpu) throw new Error("No navigator.gpu");
  await init();

  for (const [ruleIdx, rule] of getRules().entries()) {
    ruleSelect.appendChild(new Option(rule.name(), ruleIdx));
  }
  ruleSelect.addEventListener("change", () => {
    setNewRule(ruleSelect.value);
  });
  sizeSelect.addEventListener("change", () => {
    setNewSize(sizeSelect.value);
  });
  document
    .getElementById("downloadButton")
    .addEventListener("click", globalThis.downloadImage);
  document.getElementById("resetButton").addEventListener("click", () => {
    resetGame();
    canvas.focus();
  });
  document
    .getElementById("fullscreenButton")
    .addEventListener("click", toggleFullscreen);
  document
    .getElementById("hideControlsButton")
    .addEventListener("click", toggleControls);
  aboutDialog.addEventListener("close", () => {
    overlayElement.classList.remove("hidden-due-to-dialog");
  });
  document.getElementById("about-link").addEventListener("click", (event) => {
    event.preventDefault();
    overlayElement.classList.add("hidden-due-to-dialog");
    aboutDialog.showModal();
    document.getElementById("close-dialog").focus();
  });
  pauseButton.addEventListener("click", togglePause);
  densityInput.addEventListener("change", () => {
    setDensity(densityInput.value);
  });
  generationsPerSecondInput.addEventListener("change", () => {
    setGenerationsPerSecond(generationsPerSecondInput.value);
  });

  const urlParams = new URLSearchParams(window.location.hash.substring(1));
  const rule = parseInt(urlParams.get("rule"));
  const size = parseInt(urlParams.get("size"));
  const seed = parseInt(urlParams.get("seed"));
  const density = parseInt(urlParams.get("density"));
  const paused = "true" === urlParams.get("paused");
  const generationsPerSecond = parseInt(urlParams.get("gps"));

  await run(rule, size, seed, density, paused, generationsPerSecond);
} catch (e) {
  console.error("error", e);
  canvas.remove();
  overlayElement.remove();
  document.getElementById("webgpu-not-working").style.display = "block";
  document.getElementById("close-dialog").remove();
  aboutDialog.addEventListener("cancel", (e) => e.preventDefault());
  aboutDialog.showModal();
}

const registerServiceWorker = async () => {
  const digestMessage = async (message) => {
    const msgUint8 = new TextEncoder().encode(message);
    const hashBuffer = await crypto.subtle.digest("SHA-256", msgUint8);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
  };

  if ("serviceWorker" in navigator) {
    try {
      const swResponse = await fetch("/service-worker.js?ts=" + Date.now());
      const swText = await swResponse.text();
      const swHash = await digestMessage(swText);
      const registration = await navigator.serviceWorker.register(
        "/service-worker.js?hash=" + swHash,
        {
          scope: "/",
        },
      );

      // https://whatwebcando.today/articles/handling-service-worker-updates/
      registration.addEventListener("updatefound", () => {
        if (registration.installing) {
          registration.installing.addEventListener("statechange", () => {
            if (registration.waiting && navigator.serviceWorker.controller) {
              console.log("Reloading page due to service worker update");
              window.location.reload();
            }
          });
        }
      });
    } catch (error) {
      console.error(`Registration failed with ${error}`);
    }
  }
};

registerServiceWorker();
