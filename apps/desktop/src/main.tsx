import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import Overlay from "./Overlay";
import "./design-system/tokens.css";
import "./design-system/components.css";
import "./styles.css";
import "./overlay.css";
import "./settings.css";

const view = new URLSearchParams(window.location.search).get("view");
const RootView = view === "overlay" ? Overlay : App;

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RootView />
  </StrictMode>,
);
