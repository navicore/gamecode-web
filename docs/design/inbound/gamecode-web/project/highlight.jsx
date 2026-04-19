// Tiny syntax highlighter — just enough to look real for JS/TS/Python/bash/sql
function highlightJS(code) {
  const keywords = /\b(const|let|var|function|return|if|else|for|while|async|await|import|from|export|default|new|class|extends|try|catch|throw|typeof|instanceof|null|undefined|true|false)\b/g;
  const strings = /(['"`])((?:\\.|(?!\1).)*?)\1/g;
  const comments = /(\/\/[^\n]*|\/\*[\s\S]*?\*\/)/g;
  const numbers = /\b(\d+(?:_\d+)*(?:\.\d+)?)\b/g;
  const funcs = /\b([a-zA-Z_]\w*)(?=\()/g;

  // Tokenize by replacing with placeholders to avoid double-wrapping
  const placeholders = [];
  const mark = (s, cls) => {
    const i = placeholders.length;
    placeholders.push(`<span class="tok-${cls}">${s}</span>`);
    return `\u0000${i}\u0000`;
  };
  let out = code;
  out = out.replace(comments, (m) => mark(m, 'c'));
  out = out.replace(strings, (m) => mark(m, 's'));
  out = out.replace(keywords, (m) => mark(m, 'k'));
  out = out.replace(numbers, (m) => mark(m, 'n'));
  out = out.replace(funcs, (m, n) => {
    if (['if','for','while','return','switch','catch','function'].includes(n)) return m;
    return mark(n, 'f');
  });
  out = out.replace(/\u0000(\d+)\u0000/g, (_, i) => placeholders[+i]);
  return out;
}

function escapeHtml(s) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

window.highlight = (code, lang) => {
  const escaped = escapeHtml(code);
  if (lang === 'js' || lang === 'ts' || lang === 'javascript' || lang === 'typescript') {
    return highlightJS(escaped);
  }
  if (lang === 'text' || !lang) return escaped;
  // Default: just escape. Good enough for the prototype.
  return highlightJS(escaped);
};
