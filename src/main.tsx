import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "@/App";
import "@/styles/global.css";

const el = document.getElementById("root");
if (!el) throw new Error("Missing #root element");

try {
  const saved = localStorage.getItem("workset-theme");
  const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
  if (saved === "dark" || (!saved && prefersDark)) {
    document.documentElement.classList.add("dark");
  }
} catch {
  // ignore storage errors
}

createRoot(el).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
