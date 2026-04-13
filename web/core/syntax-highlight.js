import { escapeHtml } from './file-tree.js';

const KEYWORDS = {
  rust: ['Self', 'as', 'async', 'await', 'const', 'crate', 'else', 'enum', 'extern', 'false', 'fn', 'if', 'impl', 'let', 'loop', 'match', 'mod', 'mut', 'pub', 'return', 'self', 'static', 'struct', 'trait', 'true', 'type', 'unsafe', 'use', 'where'],
  javascript: ['async', 'await', 'break', 'case', 'catch', 'class', 'const', 'continue', 'default', 'else', 'export', 'extends', 'false', 'for', 'from', 'function', 'if', 'import', 'let', 'new', 'null', 'return', 'super', 'switch', 'this', 'throw', 'true', 'try', 'typeof', 'var', 'while'],
  typescript: ['abstract', 'as', 'async', 'await', 'break', 'case', 'catch', 'class', 'const', 'continue', 'default', 'else', 'enum', 'export', 'extends', 'false', 'for', 'from', 'function', 'if', 'implements', 'import', 'interface', 'let', 'module', 'new', 'null', 'private', 'protected', 'public', 'readonly', 'return', 'static', 'super', 'this', 'true', 'type', 'var'],
  kotlin: ['as', 'break', 'class', 'companion', 'data', 'else', 'false', 'for', 'fun', 'if', 'import', 'in', 'interface', 'is', 'null', 'object', 'override', 'package', 'private', 'protected', 'public', 'return', 'sealed', 'super', 'this', 'true', 'val', 'var', 'when'],
  java: ['abstract', 'boolean', 'break', 'case', 'catch', 'class', 'else', 'enum', 'extends', 'false', 'final', 'for', 'if', 'implements', 'import', 'interface', 'new', 'null', 'package', 'private', 'protected', 'public', 'return', 'static', 'super', 'switch', 'this', 'throw', 'true', 'try', 'void', 'while']
};

export const renderHighlightedCode = (filePath, text) => {
  const language = detectLanguage(filePath);

  if (language === 'json') {
    return highlightJson(text);
  }

  if (language === 'markdown') {
    return highlightMarkdown(text);
  }

  if (language === 'markup') {
    return highlightMarkup(text);
  }

  if (language) {
    return highlightCode(text, KEYWORDS[language]);
  }

  return escapeHtml(text);
};

const detectLanguage = (filePath) => {
  const normalizedPath = filePath.toLowerCase();

  if (normalizedPath.endsWith('.rs')) {
    return 'rust';
  }

  if (normalizedPath.endsWith('.ts') || normalizedPath.endsWith('.tsx')) {
    return 'typescript';
  }

  if (normalizedPath.endsWith('.js') || normalizedPath.endsWith('.jsx')) {
    return 'javascript';
  }

  if (normalizedPath.endsWith('.kt') || normalizedPath.endsWith('.kts')) {
    return 'kotlin';
  }

  if (normalizedPath.endsWith('.java')) {
    return 'java';
  }

  if (normalizedPath.endsWith('.html') || normalizedPath.endsWith('.xml')) {
    return 'markup';
  }

  if (normalizedPath.endsWith('.md') || normalizedPath.endsWith('.markdown')) {
    return 'markdown';
  }

  if (normalizedPath.endsWith('.json')) {
    return 'json';
  }

  return null;
};

const highlightCode = (text, keywords) => {
  const patterns = [
    { regex: /^\/\/.*$/, className: 'comment' },
    { regex: /^`(?:\\.|[^`])*`/, className: 'string' },
    { regex: /^"(?:\\.|[^"\\])*"/, className: 'string' },
    { regex: /^'(?:\\.|[^'\\])*'/, className: 'string' },
    { regex: /^\b\d+(?:\.\d+)?\b/, className: 'number' },
    { regex: keywordRegex(keywords), className: 'keyword' },
    { regex: /^(?:=>|==|!=|<=|>=|&&|\|\||::|[-+*/%=&|!:<>?]+)/, className: 'operator' },
    { regex: /^[{}()[\].,;:]/, className: 'punctuation' }
  ];

  return tokenize(text, patterns);
};

const highlightJson = (text) => tokenize(text, [
  { regex: /^"(?:\\.|[^"\\])*"(?=\s*:)/, className: 'property' },
  { regex: /^"(?:\\.|[^"\\])*"/, className: 'string' },
  { regex: /^\b(?:true|false|null)\b/, className: 'keyword' },
  { regex: /^-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b/, className: 'number' },
  { regex: /^[{}\[\],:]/, className: 'punctuation' }
]);

const highlightMarkdown = (text) => {
  if (/^#{1,6}\s/.test(text)) {
    return `<span class="token title">${escapeHtml(text)}</span>`;
  }

  return tokenize(text, [
    { regex: /^>\s.*/, className: 'comment' },
    { regex: /^[-*+]\s.*/, className: 'keyword' },
    { regex: /^`[^`]+`/, className: 'string' },
    { regex: /^\*\*[^*]+\*\*/, className: 'strong' },
    { regex: /^\*[^*]+\*/, className: 'emphasis' },
    { regex: /^\[[^\]]+\]\([^\)]+\)/, className: 'link' }
  ]);
};

const highlightMarkup = (text) => {
  let html = '';
  let lastIndex = 0;

  text.replace(/<[^>]+>/g, (tag, index) => {
    html += escapeHtml(text.slice(lastIndex, index));
    html += renderTag(tag);
    lastIndex = index + tag.length;
    return tag;
  });

  html += escapeHtml(text.slice(lastIndex));
  return html;
};

const renderTag = (tag) => {
  let html = '';
  let index = 0;

  if (tag.startsWith('</')) {
    html += '&lt;/';
    index = 2;
  } else {
    html += '&lt;';
    index = 1;
  }

  const tagNameMatch = /^[A-Za-z_][\w:.-]*/.exec(tag.slice(index));
  if (!tagNameMatch) {
    return escapeHtml(tag);
  }

  html += `<span class="token tag">${escapeHtml(tagNameMatch[0])}</span>`;
  index += tagNameMatch[0].length;

  while (index < tag.length) {
    const remaining = tag.slice(index);

    if (remaining === '>' || remaining === '/>') {
      html += escapeHtml(remaining);
      break;
    }

    const whitespace = /^\s+/.exec(remaining);
    if (whitespace) {
      html += escapeHtml(whitespace[0]);
      index += whitespace[0].length;
      continue;
    }

    const attrName = /^[A-Za-z_][\w:.-]*/.exec(remaining);
    if (attrName) {
      html += `<span class="token attr-name">${escapeHtml(attrName[0])}</span>`;
      index += attrName[0].length;
      continue;
    }

    const attrValue = /^(?:"(?:\\.|[^"])*"|'(?:\\.|[^'])*')/.exec(remaining);
    if (attrValue) {
      html += `<span class="token string">${escapeHtml(attrValue[0])}</span>`;
      index += attrValue[0].length;
      continue;
    }

    html += escapeHtml(remaining[0]);
    index += 1;
  }

  return html;
};

const tokenize = (text, patterns) => {
  let html = '';
  let remaining = text;

  while (remaining.length > 0) {
    let matched = false;

    for (const pattern of patterns) {
      const match = pattern.regex.exec(remaining);
      if (!match) {
        continue;
      }

      html += `<span class="token ${pattern.className}">${escapeHtml(match[0])}</span>`;
      remaining = remaining.slice(match[0].length);
      matched = true;
      break;
    }

    if (!matched) {
      html += escapeHtml(remaining[0]);
      remaining = remaining.slice(1);
    }
  }

  return html;
};

const keywordRegex = (keywords) => new RegExp(`^(?:${keywords.join('|')})\\b`);
