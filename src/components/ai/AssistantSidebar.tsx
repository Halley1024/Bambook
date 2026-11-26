import { useEffect, useRef, useState, type PointerEvent } from "react";
import { Bot, Check, ChevronDown, CircleHelp, Copy, LoaderCircle, MessageSquarePlus, PanelRightClose, PanelRightOpen, Send, X } from "lucide-react";
import { MarkdownContent } from "../content/MarkdownContent";
import { llmClient } from "../../platform/llmClient";
import { copyText } from "../../platform/clipboard";
import { useSettings } from "../../features/settings";
import { useNotification } from "../../features/notifications";
import { assistantRequestMessageLimit, buildAssistantRequest, measureAssistantRequest, type ConversationMessage } from "./assistantRequest";

import { AssistantContextBudget } from "./AssistantContextBudget";
import { assistantTextInstructions, type AssistantActionRequest } from "./assistantActions";

type AssistantSidebarProps = {
  actionRequest?: AssistantActionRequest | null;
  embedded?: boolean;
  contextText: string;
  contextLabel: string;
  emptyContextHint: string;
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onResizeStart?: (event: PointerEvent) => void;
};

type AssistantModel = {
  id: string;
  name: string;
  source: "local" | "api";
};

const assistantModelsStorageKey = "bambook.assistant.models";

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

export function AssistantSidebar({ actionRequest, embedded = false, contextText, contextLabel, emptyContextHint, collapsed, onToggleCollapsed, onResizeStart }: AssistantSidebarProps) {
  const { settings } = useSettings();
  const notification = useNotification();
  const [contextDraft, setContextDraft] = useState(contextText);
  const [contextExpanded, setContextExpanded] = useState(false);
  const [prompt, setPrompt] = useState("");
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [status, setStatus] = useState<"idle" | "loading" | "error">("idle");
  const [error, setError] = useState("");
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [usage, setUsage] = useState<number | null | undefined>();
  const [sentBudget, setSentBudget] = useState<ReturnType<typeof measureAssistantRequest> | null>(null);
  const [models, setModels] = useState(() => loadAssistantModels(settings.llmModel));
  const [selectedModelId, setSelectedModelId] = useState(`local:${settings.llmModel}`);
  const [newModel, setNewModel] = useState("");
  const [newModelSource, setNewModelSource] = useState<AssistantModel["source"]>("api");
  const [modelFormOpen, setModelFormOpen] = useState(false);
  const configuredModelRef = useRef(settings.llmModel);
  const mountedRef = useRef(true);
  const inFlightRef = useRef(false);
  const handledRequestRef = useRef<string | null>(null);
  const streamContentRef = useRef("");
  const streamReasoningRef = useRef("");
  const renderFrameRef = useRef<number | null>(null);
  const conversationRef = useRef<HTMLElement>(null);
  const followReplyRef = useRef(true);
  const copyTimerRef = useRef<ReturnType<typeof setTimeout>>();

  useEffect(() => setContextDraft(contextText), [contextText]);
  useEffect(() => {
    const previousId = `local:${configuredModelRef.current}`;
    const nextId = `local:${settings.llmModel}`;
    setModels((current) => current.some((model) => model.id === nextId) ? current : [{ id: nextId, name: settings.llmModel, source: "local" }, ...current]);
    setSelectedModelId((current) => current === previousId ? nextId : current);
    configuredModelRef.current = settings.llmModel;
  }, [settings.llmModel]);
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (renderFrameRef.current !== null) window.cancelAnimationFrame(renderFrameRef.current);
      clearTimeout(copyTimerRef.current);
    };
  }, []);
  useEffect(() => {
    const host = conversationRef.current;
    if (host && followReplyRef.current) host.scrollTop = host.scrollHeight;
  }, [messages]);

  const selectedModel = models.find((model) => model.id === selectedModelId) ?? models[0];
  const busy = status === "loading";
  const budget = busy && sentBudget ? sentBudget : measureAssistantRequest(buildAssistantRequest(messages, contextDraft, contextLabel, prompt));
  const rounds = messages.filter((message) => message.role === "user").length;

  const addModel = () => {
    if (inFlightRef.current) return;
    const name = newModel.trim();
    if (!name) return;
    const model: AssistantModel = { id: `${newModelSource}:${name}`, name, source: newModelSource };
    const next = models.some((item) => item.id === model.id) ? models : [...models, model];
    try { localStorage.setItem(assistantModelsStorageKey, JSON.stringify(next)); }
    catch { notification.error("模型列表未能保存到本机，本次仍可使用。"); }
    setModels(next);
    setSelectedModelId(model.id);
    setNewModel("");
    setModelFormOpen(false);
  };

  const copyReply = async (message: ConversationMessage) => {
    try {
      await copyText(message.content);
      if (!mountedRef.current) return;
      setCopiedId(message.id);
      clearTimeout(copyTimerRef.current);
      copyTimerRef.current = setTimeout(() => setCopiedId(null), 1800);
    } catch { notification.error("复制失败，请手动选择回答内容复制。"); }
  };

  const run = (instruction: string, source = contextDraft) => {
    const question = instruction.trim();
    if (!question || inFlightRef.current) return;
    const request = buildAssistantRequest(messages, source, contextLabel, question);
    const requestBudget = measureAssistantRequest(request);
    if (requestBudget.exceeded) {
      setError(requestBudget.count > assistantRequestMessageLimit ? "对话轮数已达上限，请新建对话后继续。" : "请求内容过长，请缩短引用文本或新建对话。");
      setStatus("error");
      return;
    }
    const userMessage: ConversationMessage = { id: crypto.randomUUID(), role: "user", content: question };
    const assistantMessage: ConversationMessage = { id: crypto.randomUUID(), role: "assistant", content: "", model: selectedModel.name, status: "streaming" };
    inFlightRef.current = true;
    followReplyRef.current = true;
    setMessages((current) => [...current, userMessage, assistantMessage]);
    setPrompt("");
    setStatus("loading");
    setError("");
    setUsage(undefined);
    setSentBudget(requestBudget);
    streamContentRef.current = "";
    streamReasoningRef.current = "";
    const flush = () => {
      if (renderFrameRef.current !== null) window.cancelAnimationFrame(renderFrameRef.current);
      renderFrameRef.current = null;
      if (!mountedRef.current) return;
      const content = streamContentRef.current;
      const reasoning = streamReasoningRef.current;
      setMessages((current) => current.map((message) => message.id === assistantMessage.id ? { ...message, content, reasoning } : message));
    };
    void llmClient.chat(settings, request, {
      onDelta: (content) => {
        if (!mountedRef.current) return;
        streamContentRef.current += content;
        if (renderFrameRef.current === null) renderFrameRef.current = window.requestAnimationFrame(flush);
      },
      onReasoning: (content) => {
        if (!mountedRef.current) return;
        streamReasoningRef.current += content;
        if (renderFrameRef.current === null) renderFrameRef.current = window.requestAnimationFrame(flush);
      },
      onComplete: flush,
    }, selectedModel.name).then((response) => {
      flush();
      if (!mountedRef.current) return;
      setMessages((current) => current.map((message) => message.id === assistantMessage.id
        ? { ...message, content: response.content || streamContentRef.current, model: response.model || selectedModel.name, status: "complete" } : message));
      setUsage(response.totalTokens);
      setStatus("idle");
    }).catch((requestError) => {
      flush();
      if (!mountedRef.current) return;
      setMessages((current) => current.map((message) => message.id === assistantMessage.id ? { ...message, content: streamContentRef.current, status: "error" } : message));
      setError(String(requestError));
      setPrompt((current) => current.trim() ? current : question);
      setStatus("error");
    }).finally(() => { inFlightRef.current = false; });
  };

  useEffect(() => {
    if (!actionRequest || handledRequestRef.current === actionRequest.id) return;
    handledRequestRef.current = actionRequest.id;
    setContextDraft(actionRequest.text);
    if (!actionRequest.action) return;
    const instruction = assistantTextInstructions[actionRequest.action];
    if (inFlightRef.current) {
      setPrompt(instruction);
      notification.info("正在生成回答，已带入新的文字与问题；完成后可点击发送。");
    } else run(instruction, actionRequest.text);
  }, [actionRequest]);

  return <aside className={`assistant-sidebar ${embedded ? "embedded" : ""} ${collapsed ? "collapsed" : ""}`} aria-label="AI 小助手">
    {collapsed ? <button className="assistant-sidebar-expand" onClick={onToggleCollapsed} data-tooltip="展开 AI 小助手" aria-label="展开 AI 小助手"><PanelRightOpen size={19} /><span>AI 小助手</span></button> : <>
      {!embedded && <header className="assistant-sidebar-header"><div><Bot size={17} /><h2>AI 小助手</h2></div><button onClick={onToggleCollapsed} data-tooltip="收起 AI 小助手" aria-label="收起 AI 小助手"><PanelRightClose size={18} /></button></header>}
      <div className="assistant-session-toolbar"><span>阅读与思考</span><button type="button" disabled={busy || !messages.length} data-tooltip="新建对话：清空当前对话，保留引用文本" onClick={() => { setMessages([]); setError(""); setStatus("idle"); setUsage(undefined); }}><MessageSquarePlus size={15} />新建对话</button></div>
      <div className="assistant-sidebar-content">
        <section className="assistant-conversation" aria-label="对话回答" ref={conversationRef} onScroll={(event) => {
          const host = event.currentTarget;
          followReplyRef.current = host.scrollHeight - host.scrollTop - host.clientHeight < 56;
        }}>
          {messages.length ? <div className="assistant-message-list">{messages.map((message) => <article className={`assistant-message ${message.role} ${message.status ?? ""}`} key={message.id}>
            <header><span>{message.role === "user" ? "你" : "AI 小助手"}</span>{message.role === "assistant" && <small title={message.model}>{message.status === "streaming" ? "正在生成…" : message.status === "error" ? "生成未完成" : message.model}</small>}</header>
            {message.role === "assistant" && message.reasoning && <details className="assistant-reasoning"><summary>{message.status === "streaming" && !message.content ? "正在思考" : "思考过程"} · 已接收 {message.reasoning.length.toLocaleString()} 字符</summary><div className="assistant-message-content plain">{message.reasoning}</div></details>}
            {message.role === "assistant" && message.status === "complete" ? <MarkdownContent className="assistant-message-content" content={message.content || "模型未返回内容。"} /> : <div className="assistant-message-content plain">{message.content || (message.status === "streaming" ? message.reasoning ? "正在思考，等待回答正文…" : "等待模型输出…" : "")}</div>}
            {message.role === "assistant" && message.content && message.status !== "streaming" && <button className="assistant-copy" type="button" onClick={() => void copyReply(message)} aria-label={copiedId === message.id ? "回答已复制" : "复制回答"}>{copiedId === message.id ? <Check size={13} /> : <Copy size={13} />}{copiedId === message.id ? "已复制" : "复制"}</button>}
          </article>)}</div> : <div className="assistant-empty"><span><Bot size={26} /></span><strong>一起读懂，也一起思考</strong><p>右键选中文字或文字批注，即可总结、解释或翻译。也可以直接输入问题。</p></div>}
        </section>
        <div className="assistant-dock">
          <section className="assistant-source-panel">
            <div className="assistant-source-heading"><button type="button" aria-expanded={contextExpanded} aria-controls="assistant-reference" onClick={() => setContextExpanded((value) => !value)}><ChevronDown size={14} className={contextExpanded ? "expanded" : ""} /><span title={contextLabel}>{contextLabel}</span><small>{contextDraft.length.toLocaleString()} 字符</small></button><button type="button" className="assistant-icon-button" aria-label="清空引用文本" data-tooltip="清空引用文本" disabled={!contextDraft || busy} onClick={() => setContextDraft("")}><X size={14} /></button></div>
            <div id="assistant-reference" hidden={!contextExpanded}><textarea aria-label={contextLabel} value={contextDraft} rows={3} placeholder={emptyContextHint} onChange={(event) => setContextDraft(event.target.value)} /></div>
            {!contextExpanded && <p className="assistant-source-preview" title={contextDraft || emptyContextHint}>{contextDraft || "选择文档文字，或展开后粘贴引用内容。"}</p>}
          </section>

          <section className="assistant-composer">
            <textarea value={prompt} aria-label="向 AI 小助手提问" rows={3} placeholder="输入问题，让思考继续…" onChange={(event) => setPrompt(event.target.value)} onKeyDown={(event) => { if (!event.nativeEvent.isComposing && (event.ctrlKey || event.metaKey) && event.key === "Enter") { event.preventDefault(); run(prompt); } }} />
            <div className="assistant-composer-controls">
              <label className="assistant-model-selector"><span className="assistant-sr-only">选择 LLM 模型</span><select aria-label="选择 LLM 模型" title={selectedModel.name} value={selectedModelId} disabled={busy} onChange={(event) => {
                if (event.target.value === "__add_model__") setModelFormOpen(true);
                else setSelectedModelId(event.target.value);
              }}>
                <optgroup label="本地模型">{models.filter((model) => model.source === "local").map((model) => <option key={model.id} value={model.id}>{model.name}</option>)}</optgroup><optgroup label="API 模型">{models.filter((model) => model.source === "api").map((model) => <option key={model.id} value={model.id}>{model.name}</option>)}</optgroup>
                <option value="__add_model__">添加模型…</option>
              </select><ChevronDown size={13} aria-hidden="true" /></label>
              <AssistantContextBudget compact bytes={budget.bytes} exceeded={budget.exceeded} messageLimitReached={budget.count > assistantRequestMessageLimit} />
              <button type="button" className="assistant-send" aria-label={busy ? "正在生成回答" : "发送问题"} data-tooltip={busy ? "正在生成回答" : "发送问题"} disabled={busy || !prompt.trim() || budget.exceeded} onClick={() => run(prompt)}>{busy ? <LoaderCircle className="assistant-spin" size={15} /> : <Send size={15} />}<span>{busy ? "生成中" : "发送"}</span></button>
            </div>
            {modelFormOpen && <div className="assistant-model-form" role="group" aria-label="添加模型">
              <label>来源<select aria-label="新增模型来源" value={newModelSource} onChange={(event) => setNewModelSource(event.target.value as AssistantModel["source"])}><option value="local">本地模型</option><option value="api">API 模型</option></select></label>
              <label>模型 ID<input autoFocus value={newModel} placeholder="输入模型 ID" onChange={(event) => setNewModel(event.target.value)} onKeyDown={(event) => { if (event.nativeEvent.isComposing) return; if (event.key === "Enter") { event.preventDefault(); addModel(); } if (event.key === "Escape") setModelFormOpen(false); }} /></label>
              <div><button type="button" onClick={() => setModelFormOpen(false)}>取消</button><button type="button" disabled={!newModel.trim()} onClick={addModel}>添加模型</button></div>
            </div>}

            {status === "error" && <p className="assistant-error" role="alert">{error}</p>}
          </section>
        </div>
      </div>
      <footer className="assistant-statusbar" aria-label="AI 助手状态">
        <div className="assistant-status-line"><span className={`assistant-run-status ${status}`} role="status">{busy ? <LoaderCircle className="assistant-spin" size={13} /> : status === "error" ? <CircleHelp size={13} /> : <Check size={13} />}{busy ? "正在生成" : status === "error" ? "请求未完成" : messages.length ? "回答已完成" : "准备就绪"}</span><span>{usage != null ? `${usage.toLocaleString()} tokens` : `${rounds} 轮对话`}</span></div>
        {budget.exceeded && <p className="assistant-budget-warning" role="status">{budget.count > assistantRequestMessageLimit ? "对话已达上限，请新建对话" : "内容过长，请缩短引用或新建对话"}</p>}
      </footer>
    </>}
    {!collapsed && onResizeStart && <div className="panel-resize-handle left-edge" role="separator" aria-orientation="vertical" aria-label="调整 AI 小助手宽度" onPointerDown={onResizeStart} />}
  </aside>;
}
