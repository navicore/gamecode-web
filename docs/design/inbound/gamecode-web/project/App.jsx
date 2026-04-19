// Main App — orchestrates state, streaming, infinite scroll, theme
const { useState: useS, useEffect: useE, useRef: useR, useCallback: useC } = React;

const TWEAK_DEFAULS = /*EDITMODE-BEGIN*/{
  "theme": "light"
}/*EDITMODE-END*/;

// Streaming simulation: break a reply into token-ish chunks
function tokenize(content) {
  // Flatten into a sequence of "steps" — each step is a partial version of content
  const steps = [];
  const acc = [];
  for (const block of content) {
    if (block.type === 'p') {
      const words = block.text.split(/(\s+)/);
      let partial = '';
      acc.push({ type: 'p', text: '' });
      const idx = acc.length - 1;
      for (const w of words) {
        partial += w;
        acc[idx] = { type: 'p', text: partial };
        steps.push(JSON.parse(JSON.stringify(acc)));
      }
    } else if (block.type === 'ul' || block.type === 'ol') {
      acc.push({ type: block.type, items: [] });
      const idx = acc.length - 1;
      for (let i = 0; i < block.items.length; i++) {
        const words = block.items[i].split(/(\s+)/);
        let partial = '';
        acc[idx].items.push('');
        for (const w of words) {
          partial += w;
          acc[idx].items[i] = partial;
          steps.push(JSON.parse(JSON.stringify(acc)));
        }
      }
    } else if (block.type === 'code') {
      acc.push({ type: 'code', lang: block.lang, code: '' });
      const idx = acc.length - 1;
      const lines = block.code.split('\n');
      let partial = '';
      for (let i = 0; i < lines.length; i++) {
        partial += (i > 0 ? '\n' : '') + lines[i];
        acc[idx].code = partial;
        steps.push(JSON.parse(JSON.stringify(acc)));
      }
    } else {
      acc.push(block);
      steps.push(JSON.parse(JSON.stringify(acc)));
    }
  }
  return steps;
}

// Canned follow-up reply for the demo
const CANNED_REPLY = {
  role: 'assistant',
  persona: 'engineer',
  model: 'qwen2.5-coder:32b',
  provider: 'ollama',
  content: [
    { type: 'p', text: "That's a reasonable plan. One more nuance worth flagging — if you go with a dedicated session-mode pool for the order service, make sure to **size it to your peak concurrency of long-held transactions**, not requests. Session pooling holds the backend for the full client session." },
    { type: 'p', text: "Rough checklist before you roll it out:" },
    { type: 'ol', items: [
      "Measure p95 transaction duration on the order service (usually the blind spot)",
      "Set `server_lifetime` to 1h so backends recycle and don't hoard memory",
      "Add a Prometheus alert on `pgbouncer_pool_waiting_client_count > 0` for more than 30s",
    ]},
    { type: 'p', text: "If you want, I can sketch the full pgbouncer config diff." },
  ],
};

function App() {
  const [theme, setTheme] = useS(TWEAK_DEFAULS.theme);
  const [conversations, setConversations] = useS(window.CONVERSATIONS);
  const [activeId, setActiveId] = useS('c1');
  const [messages, setMessages] = useS(window.RECENT_MESSAGES);
  const [olderLoaded, setOlderLoaded] = useS(false);
  const [loadingOlder, setLoadingOlder] = useS(false);
  const [model, setModel] = useS('qwen2.5-coder:32b');
  const [persona, setPersona] = useS('engineer');
  const [temp, setTemp] = useS(0.7);
  const [streaming, setStreaming] = useS(false);
  const [streamMsg, setStreamMsg] = useS(null);
  const streamRef = useR(null);
  const threadRef = useR(null);
  const pendingScrollTopRef = useR(null);

  // Theme
  useE(() => { document.documentElement.dataset.theme = theme; }, [theme]);

  // Edit-mode protocol
  useE(() => {
    const onMsg = (e) => {
      if (e.data?.type === '__activate_edit_mode') setTweaksOpen(true);
      if (e.data?.type === '__deactivate_edit_mode') setTweaksOpen(false);
    };
    window.addEventListener('message', onMsg);
    window.parent.postMessage({ type: '__edit_mode_available' }, '*');
    return () => window.removeEventListener('message', onMsg);
  }, []);
  const [tweaksOpen, setTweaksOpen] = useS(false);

  const activeConv = conversations.find(c => c.id === activeId);

  // Scroll to bottom on initial load + after send
  useE(() => {
    const el = threadRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, []);

  // After loading older, preserve scroll position
  useE(() => {
    if (pendingScrollTopRef.current !== null && threadRef.current) {
      const prev = pendingScrollTopRef.current;
      const el = threadRef.current;
      const newHeight = el.scrollHeight;
      el.scrollTop = newHeight - prev;
      pendingScrollTopRef.current = null;
    }
  }, [messages.length]);

  const loadOlder = useC(() => {
    if (olderLoaded || loadingOlder) return;
    setLoadingOlder(true);
    pendingScrollTopRef.current = threadRef.current
      ? threadRef.current.scrollHeight - threadRef.current.scrollTop
      : null;
    setTimeout(() => {
      setMessages(m => [...window.OLDER_MESSAGES, ...m]);
      setOlderLoaded(true);
      setLoadingOlder(false);
    }, 900);
  }, [olderLoaded, loadingOlder]);

  const onScroll = useC((e) => {
    if (e.target.scrollTop < 60) loadOlder();
  }, [loadOlder]);

  const persistTheme = (t) => {
    setTheme(t);
    window.parent.postMessage({ type: '__edit_mode_set_keys', edits: { theme: t } }, '*');
  };

  const handleSend = ({ text, attachments }) => {
    if (streaming) return;
    const userMsg = {
      id: 'u' + Date.now(),
      role: 'user',
      when: 'now',
      content: [{ type: 'p', text }],
      attachments,
    };
    setMessages(m => [...m, userMsg]);

    // Kick off streamed assistant reply
    setTimeout(() => startStream(), 400);
    // Scroll to bottom
    setTimeout(() => {
      if (threadRef.current) threadRef.current.scrollTop = threadRef.current.scrollHeight;
    }, 50);
  };

  const startStream = () => {
    setStreaming(true);
    const base = {
      id: 'a' + Date.now(),
      role: 'assistant',
      when: 'now',
      persona,
      model,
      provider: window.PROVIDERS.find(p => p.models.some(m => m.id === model))?.id,
      content: [],
    };
    setStreamMsg(base);

    const steps = tokenize(CANNED_REPLY.content);
    let i = 0;
    const tick = () => {
      if (i >= steps.length) {
        // Finalize
        setMessages(m => [...m, { ...base, content: CANNED_REPLY.content, when: 'just now' }]);
        setStreamMsg(null);
        setStreaming(false);
        streamRef.current = null;
        return;
      }
      setStreamMsg({ ...base, content: steps[i] });
      i++;
      // Auto-scroll while streaming
      if (threadRef.current) {
        const el = threadRef.current;
        const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 200;
        if (nearBottom) el.scrollTop = el.scrollHeight;
      }
      streamRef.current = setTimeout(tick, 18 + Math.random() * 24);
    };
    tick();
  };

  const stopStream = () => {
    if (streamRef.current) clearTimeout(streamRef.current);
    if (streamMsg) {
      setMessages(m => [...m, { ...streamMsg, when: 'stopped' }]);
    }
    setStreamMsg(null);
    setStreaming(false);
  };

  const handleNew = () => {
    const id = 'new-' + Date.now();
    const newConv = { id, title: 'New conversation', when: 'now', group: 'Today', persona };
    setConversations(c => [newConv, ...c]);
    setActiveId(id);
    setMessages([]);
    setOlderLoaded(true);
  };

  const handleSelect = (id) => {
    setActiveId(id);
    // For demo, leave messages intact — the active convo is always c1's thread
  };

  return (
    <div className="app">
      <Sidebar
        conversations={conversations}
        activeId={activeId}
        onSelect={handleSelect}
        onNew={handleNew}
      />

      <main className="main">
        <header className="chat-header">
          <div className="chat-title">
            <span>{activeConv?.title || 'New conversation'}</span>
            <span className="chat-title-meta">· {messages.length + (streamMsg ? 1 : 0)} messages</span>
          </div>
          <ModelPicker value={model} onChange={setModel}/>
          <PersonaPicker value={persona} onChange={setPersona}/>
          <button className="icon-btn" title="Share"><Icon.share/></button>
          <button className="icon-btn" title="More"><Icon.more/></button>
        </header>

        <div className="thread-wrap" ref={threadRef} onScroll={onScroll}>
          <div className="thread">
            {!olderLoaded && (
              <div className="load-older">
                {loadingOlder
                  ? <><span className="spinner"/>Loading earlier messages…</>
                  : <span style={{cursor:'pointer'}} onClick={loadOlder}>↑ Scroll to load earlier messages</span>
                }
              </div>
            )}
            {olderLoaded && (
              <div className="day-divider">Earlier today · 1:41 PM</div>
            )}

            {messages.length === 0 ? (
              <EmptyState onPick={(t) => handleSend({ text: t, attachments: [] })}/>
            ) : (
              messages.map(m => <Message key={m.id} msg={m}/>)
            )}

            {streamMsg && <Message msg={streamMsg} streaming/>}
          </div>
        </div>

        <Composer
          temp={temp}
          onTempChange={setTemp}
          onSend={handleSend}
          streaming={streaming}
          onStop={stopStream}
        />
      </main>

      <div className={`tweaks ${tweaksOpen ? 'visible' : ''}`}>
        <span className="tweaks-title">Tweaks</span>
        <div className="theme-toggle">
          <button className={`theme-opt ${theme === 'light' ? 'active' : ''}`} onClick={() => persistTheme('light')}>Light</button>
          <button className={`theme-opt ${theme === 'dark' ? 'active' : ''}`} onClick={() => persistTheme('dark')}>Dark</button>
        </div>
      </div>
    </div>
  );
}

function EmptyState({ onPick }) {
  const suggestions = [
    { label: 'Debug', text: 'Help me trace a memory leak in a Node.js worker.' },
    { label: 'Plan', text: 'Draft a 2-week scope for migrating to Postgres 16.' },
    { label: 'Explain', text: 'Compare HNSW vs IVFFlat for a 5M-vector index.' },
    { label: 'Review', text: 'Critique this TypeScript file for production readiness.' },
  ];
  return (
    <div className="empty">
      <h1>How can I help, Dani?</h1>
      <p>Pick a starter or just drop a file.</p>
      <div className="suggestions">
        {suggestions.map((s, i) => (
          <button key={i} className="suggestion" onClick={() => onPick(s.text)}>
            <div className="suggestion-label">{s.label}</div>
            <div className="suggestion-text">{s.text}</div>
          </button>
        ))}
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App/>);
