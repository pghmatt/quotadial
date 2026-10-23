import { render } from "preact";

import { App } from "./app";

// On macOS the popover sits on the system's translucent popover material; elsewhere it paints its
// own background.
document.documentElement.classList.add(navigator.userAgent.includes("Mac OS") ? "mac" : "solid");

const root = document.getElementById("app");
if (root) {
  render(<App />, root);
}
