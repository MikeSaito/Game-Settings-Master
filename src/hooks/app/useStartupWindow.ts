import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "@/lib/api/tauriRuntime";

export function useStartupWindow() {
  useEffect(() => {
    window.dispatchEvent(new Event("gsm:mounted"));
    if (!isTauriRuntime()) return;
    let revealed = false;
    const reveal = () => {
      if (revealed) return;
      revealed = true;
      void invoke("show_main_window", {
        theme: document.documentElement.dataset.theme ?? "dark",
      })
        .then(() => {
          document.documentElement.dataset.nativeReady = "true";
        })
        .catch(console.error);
    };
    // Hidden WebView2 windows can suspend animation frames; never wait on them forever.
    const fallback = window.setTimeout(reveal, 100);
    const frame = requestAnimationFrame(reveal);
    return () => {
      window.clearTimeout(fallback);
      cancelAnimationFrame(frame);
    };
  }, []);
}
