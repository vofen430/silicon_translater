import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

type TranslationRequest = {
  text: string;
  source_lang: string;
  target_lang: string;
  model: string;
};

type TranslationResponse = {
  translated_text: string;
  detected_source_lang?: string | null;
};

type SelectionEvent = {
  text: string;
  source: "UiAutomation" | "ClipboardFallback" | "OcrPlaceholder";
  bounds?: { left: number; top: number; right: number; bottom: number } | null;
};

const sourceText = document.querySelector<HTMLTextAreaElement>("#sourceText")!;
const targetText = document.querySelector<HTMLTextAreaElement>("#targetText")!;
const status = document.querySelector<HTMLParagraphElement>("#status")!;
const sourceLang = document.querySelector<HTMLSelectElement>("#sourceLang")!;
const targetLang = document.querySelector<HTMLSelectElement>("#targetLang")!;
const modelInput = document.querySelector<HTMLInputElement>("#model")!;

modelInput.value = "Qwen/Qwen2.5-7B-Instruct";

async function translate() {
  status.textContent = "翻译中...";
  const payload: TranslationRequest = {
    text: sourceText.value.trim(),
    source_lang: sourceLang.value,
    target_lang: targetLang.value,
    model: modelInput.value.trim(),
  };

  if (!payload.text) {
    status.textContent = "请输入或选中要翻译的文本";
    return;
  }

  try {
    const response = await invoke<TranslationResponse>("translate", { request: payload });
    targetText.value = response.translated_text;
    status.textContent = "翻译完成";
  } catch (error) {
    status.textContent = `翻译失败: ${String(error)}`;
  }
}

document.querySelector<HTMLButtonElement>("#translate")!.addEventListener("click", () => {
  translate();
});

document.querySelector<HTMLButtonElement>("#copyTarget")!.addEventListener("click", () => {
  targetText.select();
  document.execCommand("copy");
  status.textContent = "译文已复制";
});

document.querySelector<HTMLButtonElement>("#copySource")!.addEventListener("click", () => {
  sourceText.select();
  document.execCommand("copy");
  status.textContent = "原文已复制";
});

document.querySelector<HTMLButtonElement>("#speakText")!.addEventListener("click", () => {
  if (!targetText.value) {
    status.textContent = "无译文可朗读";
    return;
  }
  const utterance = new SpeechSynthesisUtterance(targetText.value);
  window.speechSynthesis.speak(utterance);
  status.textContent = "朗读中";
});

listen<SelectionEvent>("selection-event", (event) => {
  if (event.payload.text) {
    sourceText.value = event.payload.text;
    status.textContent = `捕获到选中文本（${event.payload.source}）`;
  }
});

listen("toggle-detection", () => {
  status.textContent = "切换划词检测";
});

listen("trigger-screenshot", () => {
  status.textContent = "截图翻译入口（待实现）";
});

listen("open-settings", () => {
  status.textContent = "打开设置";
});
