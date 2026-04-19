// Composer — multi-line input, file drop (chips for docs, thumbs for images), temperature slider
const { useState: useState_c, useRef: useRef_c, useEffect: useEffect_c } = React;

function formatSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / 1024 / 1024).toFixed(1) + ' MB';
}

function isImage(file) {
  return file.type?.startsWith('image/') || /\.(png|jpe?g|gif|webp|svg)$/i.test(file.name || '');
}

function fileExt(name) {
  const m = name.match(/\.([^.]+)$/);
  return m ? m[1].slice(0, 4) : 'FILE';
}

function Composer({ temp, onTempChange, onSend, streaming, onStop }) {
  const [text, setText] = useState_c('');
  const [attachments, setAttachments] = useState_c([]);
  const [dragging, setDragging] = useState_c(false);
  const taRef = useRef_c(null);
  const fileInputRef = useRef_c(null);

  // Auto-resize textarea
  useEffect_c(() => {
    const ta = taRef.current;
    if (!ta) return;
    ta.style.height = 'auto';
    ta.style.height = Math.min(ta.scrollHeight, 220) + 'px';
  }, [text]);

  const addFiles = (files) => {
    const next = Array.from(files).map(f => ({
      id: Math.random().toString(36).slice(2),
      name: f.name,
      size: f.size,
      isImg: isImage(f),
      preview: isImage(f) ? URL.createObjectURL(f) : null,
    }));
    setAttachments(a => [...a, ...next]);
  };

  const removeAttachment = (id) => setAttachments(a => a.filter(x => x.id !== id));

  const submit = () => {
    if (!text.trim() && !attachments.length) return;
    onSend({ text: text.trim(), attachments });
    setText('');
    setAttachments([]);
  };

  const onKeyDown = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  };

  // Global drag handlers on the composer
  const onDragEnter = (e) => { e.preventDefault(); setDragging(true); };
  const onDragOver = (e) => { e.preventDefault(); setDragging(true); };
  const onDragLeave = (e) => {
    if (e.currentTarget.contains(e.relatedTarget)) return;
    setDragging(false);
  };
  const onDrop = (e) => {
    e.preventDefault();
    setDragging(false);
    if (e.dataTransfer.files?.length) addFiles(e.dataTransfer.files);
  };

  return (
    <div className="composer-wrap">
      <div
        className={`composer ${dragging ? 'dragging' : ''}`}
        onDragEnter={onDragEnter}
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onDrop={onDrop}
      >
        {attachments.length > 0 && (
          <div className="attachments">
            {attachments.map(a => (
              <div key={a.id} className="attachment">
                {a.isImg
                  ? <div className="attachment-img" style={{backgroundImage: `url(${a.preview})`}}/>
                  : <div className="attachment-icon">{fileExt(a.name)}</div>
                }
                <div style={{minWidth: 0, flex:1}}>
                  <div className="attachment-name">{a.name}</div>
                  <div className="attachment-size">{formatSize(a.size)}</div>
                </div>
                <button className="attachment-x" onClick={() => removeAttachment(a.id)}>
                  <Icon.x/>
                </button>
              </div>
            ))}
          </div>
        )}

        <textarea
          ref={taRef}
          className="composer-input"
          placeholder="Ask anything, or drop a file…"
          value={text}
          onChange={e => setText(e.target.value)}
          onKeyDown={onKeyDown}
          rows={1}
        />

        <div className="composer-toolbar">
          <button
            className="tool-btn"
            title="Attach file"
            onClick={() => fileInputRef.current?.click()}
          >
            <Icon.paperclip/>
          </button>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            style={{display:'none'}}
            onChange={e => { if (e.target.files) addFiles(e.target.files); e.target.value=''; }}
          />
          <button className="tool-btn" title="Voice"><Icon.mic/></button>

          <label className="temp-control" title="Temperature">
            <span className="temp-label">temp</span>
            <input
              className="temp-slider"
              type="range"
              min={0} max={2} step={0.1}
              value={temp}
              onChange={e => onTempChange(parseFloat(e.target.value))}
            />
            <span className="temp-value">{temp.toFixed(1)}</span>
          </label>

          <div className="toolbar-spacer"/>

          {streaming ? (
            <button className="send-btn streaming" onClick={onStop} title="Stop">
              <Icon.stop/>
            </button>
          ) : (
            <button
              className="send-btn"
              onClick={submit}
              disabled={!text.trim() && !attachments.length}
              title="Send ⏎"
            >
              <Icon.send/>
            </button>
          )}
        </div>

        {dragging && (
          <div className="drop-overlay visible">
            <div className="drop-overlay-inner">
              <Icon.upload/>
              <div>Drop to attach</div>
            </div>
          </div>
        )}
      </div>

      <div className="composer-footnote">
        <span>Responses are local-first — streamed through <span className="kbd">/v1/chat</span></span>
        <span><span className="kbd">↵</span> send &nbsp; <span className="kbd">⇧↵</span> newline</span>
      </div>
    </div>
  );
}

window.Composer = Composer;
