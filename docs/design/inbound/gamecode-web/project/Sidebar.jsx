// Sidebar component
const { useState } = React;

function Sidebar({ conversations, activeId, onSelect, onNew }) {
  const [query, setQuery] = useState('');
  const filtered = conversations.filter(c => c.title.toLowerCase().includes(query.toLowerCase()));
  const groups = {};
  filtered.forEach(c => { (groups[c.group] = groups[c.group] || []).push(c); });
  const order = ['Today', 'Yesterday', 'Earlier'];

  return (
    <aside className="sidebar">
      <div className="sidebar-head">
        <div className="brand">
          <div className="brand-mark">GC</div>
          <span>Gamecode</span>
        </div>
        <button className="icon-btn" title="Collapse sidebar"><Icon.sidebar /></button>
      </div>

      <button className="new-chat" onClick={onNew}>
        <span style={{display:'flex', alignItems:'center', gap:8}}>
          <Icon.plus /> New chat
        </span>
        <span className="new-chat-kbd">⌘N</span>
      </button>

      <div className="search">
        <Icon.search />
        <input
          value={query}
          onChange={e => setQuery(e.target.value)}
          placeholder="Search conversations"
        />
      </div>

      <div className="conv-list">
        {order.map(g => groups[g] && (
          <div key={g}>
            <div className="conv-section-label">{g}</div>
            {groups[g].map(c => {
              const persona = window.PERSONAS.find(p => p.id === c.persona);
              return (
                <div
                  key={c.id}
                  className={`conv ${c.id === activeId ? 'active' : ''}`}
                  onClick={() => onSelect(c.id)}
                >
                  <div className="conv-dot" style={{background: persona ? persona.color : 'var(--ink-4)'}}/>
                  <div className="conv-title">{c.title}</div>
                  <div className="conv-meta">{c.when}</div>
                </div>
              );
            })}
          </div>
        ))}
      </div>

      <div className="sidebar-foot">
        <div className="user-chip">
          <div className="avatar">DC</div>
          <div>
            <div className="user-name">Dani Chen</div>
            <div className="user-status"><span className="status-dot"/>Ollama connected</div>
          </div>
        </div>
        <button className="icon-btn" title="Settings"><Icon.settings /></button>
      </div>
    </aside>
  );
}

window.Sidebar = Sidebar;
