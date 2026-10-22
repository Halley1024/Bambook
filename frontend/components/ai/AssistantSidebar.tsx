import { useEffect, useRef, useState, type CSSProperties, type PointerEvent } from "react";
import { Bot, Check, CircleHelp, Languages, PanelRightClose, PanelRightOpen, Send, Sparkles, X } from "lucide-react";
import { MarkdownContent } from "../content/MarkdownContent";
import { llmClient, type LlmMessage } from "../../platform/llmClient";
import { useSettings } from "../../features/settings";

type AssistantSidebarProps = {
  contextText: string;
  contextLabel: string;
  emptyContextHint: string;
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onResizeStart?: (event: PointerEvent) => void;
};

type ConversationMessage = {
  id: string;
  role: "user" | "assistant";
  content: string;
  model?: string;
  status?: "streaming" | "complete" | "error";
};

type AssistantModel = {
  id: string;
  name: string;
  source: "local" | "api";
};

const assistantModelsStorageKey = "bambook.assistant.models";
const contextCharacterLimit = 96_000;
const addModelOption = "__add_model__";
const systemPrompt = "你是 Bambook 的阅读助手。请使用简体中文，回答应准确、清晰，并优先依据用户提供的文档内容。";

function loadAssistantModels(defaultModel: string) {
  const configuredModel: AssistantModel = { id: `local:${defaultModel}`, name: defaultModel, source: "local" };
  try {
    const stored = JSON.parse(localStorage.getItem(assistantModelsStorageKey) ?? "null");
    if (Array.isArray(stored)) {
      const models = stored.flatMap((model): AssistantModel[] => {
        if (typeof model === "string" && model.trim()) {
          return [{ id: `api:${model}`, name: model, source: "api" }];
        }
        if (isAssistantModel(model)) return [model];
        return [];
      });
      return models.some((model) => model.id === configuredModel.id) ? models : [configuredModel, ...models];
    }
  } catch {
    // Ignore malformed browser storage and fall back to the configured model.
  }
  return [configuredModel];
}

function isAssistantModel(value: unknown): value is AssistantModel {
  return value !== null && typeof value === "object"
    && "id" in value && "name" in value && "source" in value
    && typeof value.id === "string" && typeof value.name === "string"
    && (value.source === "local" || value.source === "api");
}

export function AssistantSidebar({ contextText, contextLabel, emptyContextHint, collapsed, onToggleCollapsed, onResizeStart }: AssistantSidebarProps) {
  const { settings } = useSettings();
  const [contextDraft, setContextDraft] = useState(contextText);
  const [prompt, setPrompt] = useState("");
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [status, setStatus] = useState<"idle" | "loading" | "error">("idle");
  const [error, setError] = useState("");
  const [models, setModels] = useState(() => loadAssistantModels(settings.llmModel));
  const [selectedModelId, setSelectedModelId] = useState(`local:${settings.llmModel}`);
  const [newModel, setNewModel] = useState("");
  const [newModelSource, setNewModelSource] = useState<AssistantModel["source"]>("api");
  const [modelFormOpen, setModelFormOpen] = useState(false);
  const configuredModelRef = useRef(settings.llmModel);
  const mountedRef = useRef(true);
  const streamContentRef = useRef("");
  const renderFrameRef = useRef<number | null>(null);

  useEffect(() => setContextDraft(contextText), [contextText]);
  useEffect(() => {
    const previousId = `local:${configuredModelRef.current}`;
    const nextId = `local:${settings.llmModel}`;
    setModels((current) => current.some((model) => model.id === nextId)
      ? current
      : [{ id: nextId, name: settings.llmModel, source: "local" }, ...current]);
    setSelectedModelId((current) => current === previousId ? nextId : current);
    configuredModelRef.current = settings.llmModel;
  }, [settings.llmModel]);
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (renderFrameRef.current !== null) window.cancelAnimationFrame(renderFrameRef.current);
    };
  }, []);

  const contextUsed = systemPrompt.length + contextDraft.trim().length + prompt.trim().length
    + messages.reduce((total, message) => total + message.content.length, 0);
  const contextProgress = Math.min(1, contextUsed / contextCharacterLimit);

  const selectedModel = models.find((model) => model.id === selectedModelId) ?? models[0];

  const persistModels = (nextModels: AssistantModel[]) => {
    setModels(nextModels);
    localStorage.setItem(assistantModelsStorageKey, JSON.stringify(nextModels));
  };

  const addModel = () => {
    const name = newModel.trim();
    if (!name) return;
    const model: AssistantModel = { id: `${newModelSource}:${name}`, name, source: newModelSource };
    persistModels(models.some((item) => item.id === model.id) ? models : [...models, model]);
    setSelectedModelId(model.id);
    setNewModel("");
    setModelFormOpen(false);
  };

  const run = (instruction: string) => {
    const question = instruction.trim();
    if (!question || status === "loading") return;
    const userMessage: ConversationMessage = { id: crypto.randomUUID(), role: "user", content: question };
    const assistantMessage: ConversationMessage = { id: crypto.randomUUID(), role: "assistant", content: "", model: selectedModel.name, status: "streaming" };
    const history: LlmMessage[] = messages
      .filter((message) => message.status !== "streaming")
      .map((message) => ({ role: message.role, content: message.content }));
    const source = contextDraft.trim();
    const contextualQuestion = source ? `${question}\n\n参考文档内容（${contextLabel}）：\n---\n${source}\n---` : question;

    setMessages((current) => [...current, userMessage, assistantMessage]);
    setPrompt("");
    setStatus("loading");
    setError("");
    streamContentRef.current = "";
    if (renderFrameRef.current !== null) window.cancelAnimationFrame(renderFrameRef.current);
    renderFrameRef.current = null;

    void llmClient.chat(settings, [
      { role: "system", content: systemPrompt },
      ...history,
      { role: "user", content: contextualQuestion },
    ], {
      onDelta: (content) => {
        streamContentRef.current += content;
        if (renderFrameRef.current !== null) return;
        renderFrameRef.current = window.requestAnimationFrame(() => {
          renderFrameRef.current = null;
          if (!mountedRef.current) return;
          const output = streamContentRef.current;
          setMessages((current) => current.map((message) => message.id === assistantMessage.id ? { ...message, content: output } : message));
        });
      },
      onComplete: () => {
        if (!mountedRef.current) return;
        if (renderFrameRef.current !== null) window.cancelAnimationFrame(renderFrameRef.current);
        renderFrameRef.current = null;
        const output = streamContentRef.current;
        setMessages((current) => current.map((message) => message.id === assistantMessage.id
          ? { ...message, content: output, status: "complete" }
          : message));
        setStatus("idle");
      },
    }, selectedModel.name).catch((requestError) => {
      if (!mountedRef.current) return;
      if (renderFrameRef.current !== null) window.cancelAnimationFrame(renderFrameRef.current);
      renderFrameRef.current = null;
      setMessages((current) => current.map((message) => message.id === assistantMessage.id
        ? { ...message, status: "error" }
        : message));
      setError(String(requestError));
      setStatus("error");
    });
  };

  const chooseModel = (value: string) => {
    if (value === addModelOption) {
      setModelFormOpen(true);
      return;
    }
    setSelectedModelId(value);
  };

  return <aside className={`assistant-sidebar ${collapsed ? "collapsed" : ""}`} aria-label="AI 小助手">
    {collapsed ? <button className="assistant-sidebar-expand" onClick={onToggleCollapsed} data-tooltip="展开 AI 小助手" aria-label="展开 AI 小助手">
      <PanelRightOpen size={19} /><span>AI 小助手</span>
    </button> : <>
      <header className="assistant-sidebar-header">
        <div><Bot size={17} /><h2>AI 小助手</h2></div>
        <button onClick={onToggleCollapsed} data-tooltip="收起 AI 小助手" aria-label="收起 AI 小助手"><PanelRightClose size={18} /></button>
      </header>
      <div className="assistant-sidebar-content">
        <section className="assistant-conversation" aria-label="对话回答">
          {messages.length ? <div className="assistant-message-list">{messages.map((message) => (
            <article className={`assistant-message ${message.role} ${message.status ?? ""}`} key={message.id}>
              <header><span>{message.role === "user" ? "你" : "AI 小助手"}</span>{message.role === "assistant" && <small>{message.status === "streaming" ? "正在生成" : message.model}</small>}</header>
              {message.role === "assistant" && message.status === "complete"
                ? <MarkdownContent className="assistant-message-content" content={message.content} />
                : <div className="assistant-message-content plain">{message.content || (message.status === "streaming" ? "正在思考…" : "")}</div>}
            </article>
          ))}</div> : <div className="assistant-empty"><Bot size={24} /><strong>阅读助手</strong><p>选择文档内容后，可进行总结、解释、翻译或连续提问。</p></div>}
        </section>

        <section className="assistant-source-panel">
          <label><span>{contextLabel}</span><textarea value={contextDraft} rows={4} placeholder={emptyContextHint} onChange={(event) => setContextDraft(event.target.value)} /></label>
        </section>

        <section className="assistant-composer">
          <textarea value={prompt} rows={3} placeholder="输入问题继续对话…" onChange={(event) => setPrompt(event.target.value)}
            onKeyDown={(event) => { if ((event.ctrlKey || event.metaKey) && event.key === "Enter") run(prompt); }} />
          {modelFormOpen && <div className="assistant-model-form">
            <select aria-label="新增模型来源" value={newModelSource} onChange={(event) => setNewModelSource(event.target.value as AssistantModel["source"])}>
              <option value="local">本地模型</option>
              <option value="api">API 模型</option>
            </select>
            <input autoFocus value={newModel} placeholder="输入模型 ID" onChange={(event) => setNewModel(event.target.value)}
              onKeyDown={(event) => { if (event.key === "Enter") addModel(); if (event.key === "Escape") setModelFormOpen(false); }} />
            <button type="button" aria-label="确认添加模型" data-tooltip="确认添加" onClick={addModel}><Check size={15} /></button>
            <button type="button" aria-label="取消添加模型" data-tooltip="取消" onClick={() => setModelFormOpen(false)}><X size={15} /></button>
          </div>}
          <div className="assistant-composer-actions">
            <div className="assistant-quick-actions">
              <button type="button" disabled={status === "loading" || !contextDraft.trim()} onClick={() => run("请概括这段文字的核心内容，并列出关键要点。")}><Sparkles size={14} />总结</button>
              <button type="button" disabled={status === "loading" || !contextDraft.trim()} onClick={() => run("请解释这段文字中的核心概念、术语和论证关系。")}><CircleHelp size={14} />解释</button>
              <button type="button" disabled={status === "loading" || !contextDraft.trim()} onClick={() => run("请将这段文字翻译为简体中文，并保留关键术语。")}><Languages size={14} />翻译</button>
            </div>
            <div className="assistant-composer-controls">
              <select aria-label="选择 LLM 模型" value={selectedModelId} disabled={status === "loading"} onChange={(event) => chooseModel(event.target.value)}>
                <optgroup label="本地模型">{models.filter((model) => model.source === "local").map((model) => <option key={model.id} value={model.id}>{model.name}</option>)}</optgroup>
                <optgroup label="API 模型">{models.filter((model) => model.source === "api").map((model) => <option key={model.id} value={model.id}>{model.name}</option>)}</optgroup>
                <option value={addModelOption}>添加模型...</option>
              </select>
              <ContextBudgetRing used={contextUsed} progress={contextProgress} />
              <button type="button" className="assistant-send" aria-label="发送问题" data-tooltip="发送问题" disabled={status === "loading" || !prompt.trim()} onClick={() => run(prompt)}><Send size={16} /></button>
            </div>
          </div>
          {status === "error" && <p className="assistant-error">{error}</p>}
        </section>
      </div>
    </>}
    {!collapsed && onResizeStart && <div className="panel-resize-handle left-edge" role="separator" aria-orientation="vertical"
      aria-label="调整 AI 小助手宽度" onPointerDown={onResizeStart} />}
  </aside>;
}

function ContextBudgetRing({ used, progress }: { used: number; progress: number }) {
  const style = { "--context-progress": `${progress * 100}%` } as CSSProperties;
  return <div className="assistant-context-budget" aria-label={`上下文已使用 ${used} / ${contextCharacterLimit} 字符`}>
    <div className="assistant-context-ring" style={style}><span>{Math.round(progress * 100)}%</span></div>
    <small>{formatContextCount(used)} / 96k</small>
  </div>;
}

function formatContextCount(value: number) {
  return value >= 1000 ? `${(value / 1000).toFixed(value >= 10_000 ? 0 : 1)}k` : String(value);
}
