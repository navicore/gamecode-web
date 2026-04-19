// Small SVG icon set
const Icon = {
  search: () => (<svg viewBox="0 0 16 16" fill="none"><circle cx="7" cy="7" r="5" stroke="currentColor" strokeWidth="1.5"/><path d="M11 11l3 3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/></svg>),
  plus: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M8 3v10M3 8h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/></svg>),
  chevDown: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M4 6l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/></svg>),
  check: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M3 8l3 3 7-7" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/></svg>),
  copy: () => (<svg viewBox="0 0 16 16" fill="none"><rect x="5" y="5" width="8" height="8" rx="1.5" stroke="currentColor" strokeWidth="1.3"/><path d="M11 5V4a1.5 1.5 0 0 0-1.5-1.5h-5A1.5 1.5 0 0 0 3 4v5A1.5 1.5 0 0 0 4.5 10.5h1" stroke="currentColor" strokeWidth="1.3"/></svg>),
  refresh: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M13 8a5 5 0 1 1-1.5-3.5M13 3v2.5h-2.5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round"/></svg>),
  thumbUp: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M6 7V4.5a1.5 1.5 0 0 1 3 0V7h2.5a1.5 1.5 0 0 1 1.5 1.7l-.5 3A1.5 1.5 0 0 1 11 13H6V7zm0 0H3.5v6H6" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round"/></svg>),
  share: () => (<svg viewBox="0 0 16 16" fill="none"><circle cx="4" cy="8" r="1.5" stroke="currentColor" strokeWidth="1.3"/><circle cx="12" cy="4" r="1.5" stroke="currentColor" strokeWidth="1.3"/><circle cx="12" cy="12" r="1.5" stroke="currentColor" strokeWidth="1.3"/><path d="M5.5 7l5-2.5M5.5 9l5 2.5" stroke="currentColor" strokeWidth="1.3"/></svg>),
  send: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M8 13V3M3.5 7.5L8 3l4.5 4.5" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round"/></svg>),
  stop: () => (<svg viewBox="0 0 16 16" fill="none"><rect x="4" y="4" width="8" height="8" rx="1.5" fill="currentColor"/></svg>),
  paperclip: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M13 7.5L7.5 13a3 3 0 0 1-4.2-4.2L9 3.1a2 2 0 0 1 2.8 2.8L6.1 11.6a1 1 0 0 1-1.4-1.4L10 4.9" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round"/></svg>),
  mic: () => (<svg viewBox="0 0 16 16" fill="none"><rect x="6" y="2" width="4" height="7" rx="2" stroke="currentColor" strokeWidth="1.3"/><path d="M4 8a4 4 0 0 0 8 0M8 12v2" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/></svg>),
  settings: () => (<svg viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="2" stroke="currentColor" strokeWidth="1.3"/><path d="M8 1.5v1.5M8 13v1.5M1.5 8H3M13 8h1.5M3.5 3.5L4.5 4.5M11.5 11.5L12.5 12.5M3.5 12.5L4.5 11.5M11.5 4.5L12.5 3.5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/></svg>),
  upload: () => (<svg viewBox="0 0 32 32" fill="none"><path d="M16 21V7M10 13l6-6 6 6M6 22v3a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-3" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"/></svg>),
  x: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/></svg>),
  sparkle: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M8 2l1.5 4.5L14 8l-4.5 1.5L8 14l-1.5-4.5L2 8l4.5-1.5L8 2z" fill="currentColor"/></svg>),
  pencil: () => (<svg viewBox="0 0 16 16" fill="none"><path d="M11 3l2 2-7 7-3 1 1-3 7-7z" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round"/></svg>),
  more: () => (<svg viewBox="0 0 16 16" fill="none"><circle cx="4" cy="8" r="1.2" fill="currentColor"/><circle cx="8" cy="8" r="1.2" fill="currentColor"/><circle cx="12" cy="8" r="1.2" fill="currentColor"/></svg>),
  sidebar: () => (<svg viewBox="0 0 16 16" fill="none"><rect x="2" y="3" width="12" height="10" rx="1.5" stroke="currentColor" strokeWidth="1.3"/><path d="M6 3v10" stroke="currentColor" strokeWidth="1.3"/></svg>),
};

window.Icon = Icon;
