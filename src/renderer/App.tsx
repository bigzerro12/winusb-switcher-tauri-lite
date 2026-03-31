import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useProbeStore } from "./store/probeStore";
import Dashboard from "./pages/Dashboard";
import { COMMANDS } from "@shared/types";

const BODY_PADDING = 24; // 12px top + 12px bottom (matches body padding in styles.css)

// Resize only the window HEIGHT to fit content; preserves whatever width the user has set.
async function resizeHeightToContent() {
  await new Promise<void>((r) => setTimeout(r, 80)); // wait one paint for DOM to settle
  const card =
    document.querySelector<HTMLElement>(".app-card") ??
    document.querySelector<HTMLElement>(".message-card") ??
    document.querySelector<HTMLElement>(".bootstrap-lite-card");
  if (!card) return;
  const targetHeight = card.scrollHeight + BODY_PADDING;
  try {
    const win = getCurrentWindow();
    const [physicalSize, scale] = await Promise.all([win.outerSize(), win.scaleFactor()]);
    const currentLogicalWidth = Math.round(physicalSize.width / scale);
    await win.setSize(new LogicalSize(currentLogicalWidth, targetHeight));
  } catch (e) {
    console.error("[App] window resize failed:", e);
  }
}

export default function App() {
  const { isInstalled, checkInstallation } = useProbeStore();
  const [bootstrap, setBootstrap] = useState<"pending" | "ok" | "error">("pending");
  const [bootstrapError, setBootstrapError] = useState<string>("");

  const runBootstrap = useCallback(async () => {
    setBootstrap("pending");
    setBootstrapError("");
    try {
      await invoke<string>(COMMANDS.PREPARE_BUNDLED_JLINK);
      setBootstrap("ok");
    } catch (err) {
      setBootstrap("error");
      setBootstrapError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  useEffect(() => {
    runBootstrap();
  }, [runBootstrap]);

  useEffect(() => {
    if (bootstrap !== "ok") return;
    checkInstallation().catch((err) => {
      console.error("[App] checkInstallation failed:", err);
    });
  }, [bootstrap, checkInstallation]);

  useEffect(() => {
    resizeHeightToContent();
  }, [bootstrap, isInstalled]);

  if (bootstrap === "pending") {
    return (
      <div className="flex items-center justify-center min-h-screen bg-white p-6">
        <div
          className="bootstrap-lite-card max-w-md text-center rounded-lg border border-[#e9ecef] bg-[#f8f9fa] p-6 shadow-sm"
        >
          <div className="text-[#495057] font-semibold mb-2">Preparing WinUSB Switcher Lite…</div>
          <p className="text-[13px] text-[#6c757d] leading-relaxed mb-4">
            This app ships with bundled SEGGER J-Link software. On first launch it is unpacked to your
            profile (<code className="text-xs bg-white px-1 py-0.5 rounded">%AppData%\Roaming\SEGGER\JLink_V930a</code>).
            This step can take a short while; please wait.
          </p>
          <div className="text-[12px] text-[#adb5bd]">Extracting files…</div>
        </div>
      </div>
    );
  }

  if (bootstrap === "error") {
    return (
      <div className="flex items-center justify-center min-h-screen bg-white p-6">
        <div className="bootstrap-lite-card max-w-md text-center rounded-lg border border-[#ffcccc] bg-[#fff5f5] p-6">
          <div className="text-[#721c24] font-semibold mb-2">Could not prepare bundled J-Link</div>
          <p className="text-[13px] text-[#721c24] mb-4 break-words">{bootstrapError}</p>
          <button type="button" className="btn btn-primary" onClick={() => void runBootstrap()}>
            Try again
          </button>
        </div>
      </div>
    );
  }

  if (isInstalled === null) {
    return (
      <div className="flex items-center justify-center h-screen bg-white">
        <div className="text-gray-400 text-sm">Checking J-Link installation...</div>
      </div>
    );
  }

  return <Dashboard />;
}
