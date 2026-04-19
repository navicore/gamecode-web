// Model and Persona popover pickers
const { useState: useState_p, useRef: useRef_p, useEffect: useEffect_p } = React;

function usePopover() {
  const [open, setOpen] = useState_p(false);
  const ref = useRef_p(null);
  useEffect_p(() => {
    if (!open) return;
    const onDown = (e) => {
      if (ref.current && !ref.current.contains(e.target)) setOpen(false);
    };
    setTimeout(() => document.addEventListener('mousedown', onDown), 0);
    return () => document.removeEventListener('mousedown', onDown);
  }, [open]);
  return { open, setOpen, ref };
}

function ModelPicker({ value, onChange }) {
  const { open, setOpen, ref } = usePopover();
  const [query, setQuery] = useState_p('');

  const selectedProvider = window.PROVIDERS.find(p => p.models.some(m => m.id === value));
  const selectedModel = selectedProvider?.models.find(m => m.id === value);

  return (
    <div ref={ref} style={{position:'relative'}}>
      <button className="pill" onClick={() => setOpen(o => !o)}>
        <span className="dot" style={{background: 'oklch(0.7 0.15 145)'}}/>
        <span style={{fontFamily:'var(--font-mono)', fontSize:'12px'}}>{selectedModel?.name}</span>
        <span className="provider-badge">{selectedProvider?.name}</span>
        <Icon.chevDown />
      </button>

      {open && (
        <div className="popover model-popover" style={{top:'calc(100% + 6px)', left:0}}>
          <div className="popover-search">
            <Icon.search />
            <input
              autoFocus
              value={query}
              onChange={e => setQuery(e.target.value)}
              placeholder="Search models…"
            />
          </div>
          <div className="popover-body">
            {window.PROVIDERS.map(p => {
              const models = p.models.filter(m => m.name.toLowerCase().includes(query.toLowerCase()));
              if (!models.length) return null;
              return (
                <div key={p.id} className="provider-group">
                  <div className="provider-label">
                    <span>{p.name}</span>
                    <span style={{color:'var(--ink-4)', fontWeight:400, textTransform:'none', letterSpacing:0, fontFamily:'var(--font-mono)', fontSize:'10px'}}>{p.desc}</span>
                    <span className="health"/>
                  </div>
                  {models.map(m => (
                    <div
                      key={m.id}
                      className={`model-row ${m.id === value ? 'selected' : ''}`}
                      onClick={() => { onChange(m.id); setOpen(false); }}
                    >
                      <div className="model-info">
                        <div className="model-name">{m.name}</div>
                        <div className="model-desc">{m.desc}</div>
                      </div>
                      <div className="model-tags">
                        {m.tags.map(t => <span key={t} className="model-tag">{t}</span>)}
                      </div>
                      {m.id === value && <span className="model-check"><Icon.check/></span>}
                    </div>
                  ))}
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

function PersonaPicker({ value, onChange }) {
  const { open, setOpen, ref } = usePopover();
  const selected = window.PERSONAS.find(p => p.id === value);

  return (
    <div ref={ref} style={{position:'relative'}}>
      <button className="pill" onClick={() => setOpen(o => !o)}>
        <span className="dot" style={{background: selected.color}}/>
        <span>{selected.name}</span>
        <Icon.chevDown />
      </button>

      {open && (
        <div className="popover persona-popover" style={{top:'calc(100% + 6px)', left:0}}>
          <div style={{padding:'10px 12px 6px', fontSize:'10.5px', textTransform:'uppercase', letterSpacing:'0.08em', color:'var(--ink-4)', fontFamily:'var(--font-mono)'}}>
            Persona · System prompt
          </div>
          {window.PERSONAS.map(p => (
            <div
              key={p.id}
              className={`persona-row ${p.id === value ? 'selected' : ''}`}
              onClick={() => { onChange(p.id); setOpen(false); }}
            >
              <span className="persona-swatch" style={{background: p.color}}/>
              <div className="persona-info">
                <div className="persona-name">{p.name}</div>
                <div className="persona-desc">{p.desc}</div>
              </div>
              {p.id === value && <span className="model-check" style={{marginTop:4}}><Icon.check/></span>}
            </div>
          ))}
          <div style={{padding:'8px 12px', borderTop:'1px solid var(--border)', display:'flex', alignItems:'center', gap:6, fontSize:'12px', color:'var(--ink-3)', cursor:'pointer'}}>
            <Icon.pencil/>
            <span>Edit personas</span>
          </div>
        </div>
      )}
    </div>
  );
}

window.ModelPicker = ModelPicker;
window.PersonaPicker = PersonaPicker;
