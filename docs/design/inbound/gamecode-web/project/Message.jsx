// Message rendering with markdown-ish content + code blocks + streaming cursor
const { useState: useState_m } = React;

function CodeBlock({ lang, code }) {
  const [copied, setCopied] = useState_m(false);
  const onCopy = () => {
    navigator.clipboard?.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 1400);
  };
  return (
    <div className="code-block">
      <div className="code-head">
        <span>{lang || 'text'}</span>
        <button className="code-copy" onClick={onCopy}>
          <Icon.copy/>
          {copied ? 'Copied' : 'Copy'}
        </button>
      </div>
      <pre><code dangerouslySetInnerHTML={{__html: window.highlight(code, lang)}}/></pre>
    </div>
  );
}

function renderInline(text) {
  // Bold **x** and inline `code`. Keep simple.
  const parts = [];
  const regex = /(\*\*[^*]+\*\*|`[^`]+`)/g;
  let last = 0;
  let m;
  let i = 0;
  while ((m = regex.exec(text))) {
    if (m.index > last) parts.push(text.slice(last, m.index));
    const tok = m[0];
    if (tok.startsWith('**')) parts.push(<strong key={i++}>{tok.slice(2, -2)}</strong>);
    else parts.push(<code key={i++}>{tok.slice(1, -1)}</code>);
    last = m.index + tok.length;
  }
  if (last < text.length) parts.push(text.slice(last));
  return parts;
}

function MessageContent({ content, streaming }) {
  return (
    <div className="msg-content">
      {content.map((b, i) => {
        const isLast = i === content.length - 1;
        const cursor = streaming && isLast ? <span className="streaming-cursor"/> : null;
        if (b.type === 'p') return <p key={i}>{renderInline(b.text)}{cursor}</p>;
        if (b.type === 'ul') return <ul key={i}>{b.items.map((it, j) => <li key={j}>{renderInline(it)}</li>)}</ul>;
        if (b.type === 'ol') return <ol key={i}>{b.items.map((it, j) => <li key={j}>{renderInline(it)}</li>)}</ol>;
        if (b.type === 'code') return <CodeBlock key={i} lang={b.lang} code={b.code}/>;
        if (b.type === 'h3') return <h3 key={i}>{b.text}</h3>;
        return null;
      })}
    </div>
  );
}

function Message({ msg, streaming }) {
  const persona = msg.persona ? window.PERSONAS.find(p => p.id === msg.persona) : null;
  const isUser = msg.role === 'user';

  return (
    <div className="msg">
      <div className="msg-rail">
        <div className={`msg-avatar ${isUser ? 'user' : 'assistant'}`}>
          {isUser ? 'DC' : (
            <span style={{color: persona?.color || 'var(--ink)'}}>
              <Icon.sparkle/>
            </span>
          )}
        </div>
        {!isUser && persona && (
          <div className="persona-line" style={{background: persona.color}}/>
        )}
      </div>
      <div className="msg-body">
        <div className="msg-head">
          <span className="msg-author">{isUser ? 'You' : (persona?.name || 'Assistant')}</span>
          {!isUser && msg.model && (
            <span className="msg-persona-tag">
              <span className="dot" style={{background: 'oklch(0.7 0.15 145)'}}/>
              {msg.model}
            </span>
          )}
          <span className="msg-meta">{msg.when}</span>
        </div>
        <MessageContent content={msg.content} streaming={streaming}/>
        {!isUser && !streaming && (
          <div className="msg-actions">
            <button className="msg-action"><Icon.copy/>Copy</button>
            <button className="msg-action"><Icon.refresh/>Regenerate</button>
            <button className="msg-action"><Icon.thumbUp/></button>
            <button className="msg-action"><Icon.share/></button>
          </div>
        )}
      </div>
    </div>
  );
}

window.Message = Message;
