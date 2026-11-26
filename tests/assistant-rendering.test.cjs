const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const ts = require("typescript");
const React = require("react");
const { renderToStaticMarkup } = require("react-dom/server");

// Compile the actual components in memory without adding a test framework.
function loadSource(relativePath, mocks = {}, cache = new Map()) {
  const filename = path.resolve(__dirname, "..", relativePath);
  if (cache.has(filename)) return cache.get(filename).exports;
  const module = { exports: {} };
  cache.set(filename, module);
  const compiled = ts.transpileModule(fs.readFileSync(filename, "utf8"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX, target: ts.ScriptTarget.ES2020 },
  }).outputText;
  const localRequire = (specifier) => {
    if (Object.hasOwn(mocks, specifier)) return mocks[specifier];
    if (!specifier.startsWith(".")) return require(specifier);
    const base = path.resolve(path.dirname(filename), specifier);
    const dependency = [base, `${base}.ts`, `${base}.tsx`].find((candidate) => fs.existsSync(candidate) && fs.statSync(candidate).isFile());
    return loadSource(dependency, mocks, cache);
  };
  new Function("require", "module", "exports", compiled)(localRequire, module, module.exports);
  return module.exports;
}

for (const usage of [null, undefined, 0, 1234]) {
  test(`assistant renders token usage ${String(usage)} without crashing`, () => {
    const { AssistantSidebar } = loadSource("src/components/ai/AssistantSidebar.tsx", {
      react: { ...React, useState: (initial) => React.useState(initial === undefined ? usage : initial) },
      "../../platform/llmClient": { llmClient: {} },
      "../../platform/clipboard": {},
      "../../features/settings": { useSettings: () => ({ settings: { llmModel: "test-model" } }) },
      "../../features/notifications": { useNotification: () => ({}) },
    });
    const html = renderToStaticMarkup(React.createElement(AssistantSidebar, {
      contextText: "", contextLabel: "引用", emptyContextHint: "", collapsed: false, onToggleCollapsed() {},
    }));
    assert.ok(html.includes(usage == null ? "0 轮对话" : `${usage.toLocaleString()} tokens`));
  });
}

test("Markdown handles incomplete block markers without stalling subsequent content", () => {
  const { MarkdownContent } = loadSource("src/components/content/MarkdownContent.tsx");
  for (const marker of ["```js`", "````", "### ", "#\t"]) {
    const html = renderToStaticMarkup(React.createElement(MarkdownContent, { content: `${marker}\n\n后续内容` }));
    assert.ok(html.includes("后续内容"), marker);
  }
});

test("Markdown still renders headings, lists, paragraphs and fenced code", () => {
  const { MarkdownContent } = loadSource("src/components/content/MarkdownContent.tsx");
  const html = renderToStaticMarkup(React.createElement(MarkdownContent, {
    content: "# 标题\n第一行\n第二行\n\n- 项目\n\n```js\nconst x = 1;\n```",
  }));
  assert.ok(html.includes("<h1>标题</h1>"));
  assert.ok(html.includes("<p>第一行<br/>第二行</p>"));
  assert.ok(html.includes("<ul><li>项目</li></ul>"));
  assert.ok(html.includes('<pre><code data-language="js">const x = 1;</code></pre>'));
});

test("reasoning reaches its handler before the request completes", async () => {
  let channel;
  let resolveRequest;
  const { llmClient } = loadSource("src/platform/llmClient.ts", {
    "@tauri-apps/api/core": { Channel: class { constructor() { channel = this; } } },
    "./tauriClient": { invokeCommand: () => new Promise((resolve) => { resolveRequest = resolve; }) },
  });
  const received = [];
  const pending = llmClient.chat({ llmModel: "test" }, [], {
    onDelta: (text) => received.push(["answer", text]),
    onReasoning: (text) => received.push(["reasoning", text]),
    onComplete: () => received.push(["complete"]),
  });
  channel.onmessage({ type: "reasoning", content: "正在分析" });
  assert.deepEqual(received, [["reasoning", "正在分析"]]);
  channel.onmessage({ type: "delta", content: "回答正文" });
  channel.onmessage({ type: "complete" });
  resolveRequest({ content: "回答正文" });
  await pending;
  assert.deepEqual(received, [["reasoning", "正在分析"], ["answer", "回答正文"], ["complete"]]);
});

test("assistant shows reasoning progress while answer content is still empty", () => {
  const messages = [{ id: "a", role: "assistant", status: "streaming", content: "", reasoning: "正在分析" }];
  const { AssistantSidebar } = loadSource("src/components/ai/AssistantSidebar.tsx", {
    react: { ...React, useState: (initial) => React.useState(Array.isArray(initial) ? messages : initial === "idle" ? "loading" : initial) },
    "../../platform/llmClient": { llmClient: {} },
    "../../platform/clipboard": {},
    "../../features/settings": { useSettings: () => ({ settings: { llmModel: "test" } }) },
    "../../features/notifications": { useNotification: () => ({}) },
  });
  const html = renderToStaticMarkup(React.createElement(AssistantSidebar, {
    contextText: "", contextLabel: "引用", emptyContextHint: "", collapsed: false, onToggleCollapsed() {},
  }));
  assert.ok(html.includes("已接收 4 字符"));
  assert.ok(html.includes("正在分析"));
  assert.ok(html.includes("正在思考，等待回答正文…"));
});

test("conversation history sends answer text without separate reasoning", () => {
  const { buildAssistantRequest } = loadSource("src/components/ai/assistantRequest.ts");
  const request = buildAssistantRequest([
    { id: "u", role: "user", content: "问题" },
    { id: "a", role: "assistant", status: "complete", content: "回答正文", reasoning: "不发送的思考过程" },
  ], "", "引用", "下一问");
  assert.ok(request.some((message) => message.content === "回答正文"));
  assert.ok(!JSON.stringify(request).includes("不发送的思考过程"));
});
