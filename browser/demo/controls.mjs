import { requiredMethod } from './addons.mjs';
import { themes } from './themes.mjs';

const choices = {
  cursorStyle: ['block', 'underline', 'bar'], cursorInactiveStyle: ['outline', 'block', 'bar', 'underline', 'none'],
  fontWeight: ['normal', 'bold', '100', '200', '300', '400', '500', '600', '700', '800', '900'],
  fontWeightBold: ['normal', 'bold', '100', '200', '300', '400', '500', '600', '700', '800', '900'],
  logLevel: ['trace', 'debug', 'info', 'warn', 'error', 'off']
};
const hidden = new Set(['cols', 'rows', 'convertEol', 'termName', 'documentOverride', 'linkHandler', 'logger']);
const nested = ['scrollbar.showScrollbar', 'scrollbar.showArrows', 'scrollbar.overviewRuler.showTopBorder',
  'scrollbar.overviewRuler.showBottomBorder', 'vtExtensions.kittyKeyboard',
  'vtExtensions.kittySgrBoldFaintControl', 'vtExtensions.win32InputMode'];
// Values preserved from the original demo/client/client.ts search controls.
const searchDecorations = Object.freeze({ matchBackground: '#232422', matchBorder: '#555753',
  matchOverviewRuler: '#555753', activeMatchBackground: '#ef2929', activeMatchBorder: '#ffffff',
  activeMatchColorOverviewRuler: '#ef2929' });
const byId = id => document.getElementById(id);
const write = (terminal, text) => new Promise((resolve, reject) => {
  try { requiredMethod(terminal, 'write')(text, resolve); } catch (error) { reject(error); }
});

function setNested(options, path, value) {
  const [parent, child, leaf] = path.split('.');
  if (leaf) options[parent] = { ...options[parent], [child]: { ...options[parent]?.[child], [leaf]: value } };
  else options[parent] = { ...options[parent], [child]: value };
}
function getNested(options, path) { return path.split('.').reduce((value, key) => value?.[key], options); }

async function copyHtml(html) {
  if (navigator.clipboard?.write && globalThis.ClipboardItem) {
    await navigator.clipboard.write([new ClipboardItem({ 'text/html': new Blob([html], { type: 'text/html' }) })]);
    return;
  }
  const copy = event => { event.clipboardData.setData('text/html', html); event.preventDefault(); };
  document.addEventListener('copy', copy);
  try { if (!document.execCommand('copy')) throw new Error('Browser rejected copying HTML'); }
  finally { document.removeEventListener('copy', copy); }
}

export function setupControls(app) {
  const controller = new AbortController();
  let optionController;
  let autoResize = false;
  function on(id, event, action) {
    byId(id).addEventListener(event, value => {
      if (event === 'submit') value.preventDefault();
      app.action(id, () => action(value));
    }, { signal: controller.signal });
  }
  function fit() {
    requiredMethod(app.addons().ensure('fit'), 'fit')();
    app.refresh();
  }
  function applyTheme() {
    const terminal = app.terminal();
    const theme = { ...themes[byId('theme').value] };
    if (terminal.options.allowTransparency) theme.background = 'transparent';
    terminal.options.theme = theme;
    byId('terminal-container').style.background = theme.background ?? '#000000';
    app.refresh();
  }
  function applyOption(key, value) {
    app.terminal().options[key] = value;
    if (key === 'allowTransparency') applyTheme();
    if (autoResize) fit();
    app.refresh();
  }
  function rebuildOptions() {
    optionController?.abort();
    optionController = new AbortController();
    const options = app.terminal().options;
    const container = byId('options');
    container.replaceChildren();
    function control(key, value, isNested) {
      const label = document.createElement('label');
      label.className = 'option';
      const name = document.createElement('span'); name.textContent = key; label.append(name);
      const input = document.createElement(choices[key] ? 'select' : 'input');
      input.id = `opt-${key.replaceAll('.', '-')}`;
      if (choices[key]) {
        for (const choice of choices[key]) {
          const option = document.createElement('option'); option.value = choice; option.textContent = choice;
          input.append(option);
        }
        input.value = String(value);
      } else if (typeof value === 'boolean' || isNested) {
        input.type = 'checkbox'; input.checked = value ?? false;
      } else if (typeof value === 'number') {
        input.type = 'number'; input.value = String(value);
        input.step = ['lineHeight', 'scrollSensitivity'].includes(key) ? '0.1' : '1';
      } else { input.type = 'text'; input.value = value; }
      input.addEventListener('change', () => app.action(`option:${key}`, () => {
        let changed = input.type === 'checkbox' ? input.checked : input.value;
        if (typeof value === 'number') changed = ['lineHeight', 'scrollSensitivity'].includes(key) ? parseFloat(changed) : parseInt(changed, 10);
        if (isNested) setNested(app.terminal().options, key, changed);
        else applyOption(key, changed);
        app.refresh();
      }), { signal: optionController.signal });
      label.append(input); container.append(label);
    }
    for (const key of Object.getOwnPropertyNames(options)) {
      const value = options[key];
      if (!hidden.has(key) && ['boolean', 'number', 'string'].includes(typeof value)) control(key, value, false);
    }
    for (const path of nested) control(path, getNested(options, path), true);
  }
  on('size-form', 'submit', () => {
    const dimensions = byId('size').value.match(/^([0-9]+)x([0-9]+)$/);
    if (dimensions) {
      autoResize = false;
      app.terminal().resize(Number(dimensions[1]), Number(dimensions[2]));
    } else { autoResize = true; byId('size').value = 'auto'; fit(); }
    app.terminal().screenElement.style.outline = autoResize ? '' : 'solid thin red';
    app.refresh();
  });
  on('padding', 'change', () => {
    const value = Number.parseInt(byId('padding').value, 10);
    if (!Number.isFinite(value) || value < 0) throw new Error('Padding must be a nonnegative integer');
    app.terminal().element.style.padding = `${value}px`;
    fit();
  });
  on('theme', 'change', applyTheme);
  on('renderer', 'change', () => {
    const addons = app.addons();
    if (byId('renderer').value === 'webgl') addons.ensure('webgl');
    else if (addons.names().includes('webgl')) addons.unload('webgl');
    app.refresh();
  });
  function search(previous) {
    const addon = app.addons().ensure('search');
    const result = requiredMethod(addon, previous ? 'findPrevious' : 'findNext')(byId('search-text').value, {
      regex: byId('search-regex').checked, caseSensitive: byId('search-case').checked,
      wholeWord: byId('search-word').checked,
      decorations: byId('search-highlight').checked ? searchDecorations : undefined
    });
    byId('search-results').textContent = JSON.stringify({ found: result,
      selection: requiredMethod(app.terminal(), 'getSelection')(), result: app.searchResults });
    app.refresh();
  }
  on('find-next', 'click', () => search(false));
  on('find-previous', 'click', () => search(true));
  on('serialize', 'click', async () => {
    const output = requiredMethod(app.addons().ensure('serialize'), 'serialize')();
    byId('serialize-output').textContent = JSON.stringify(output);
    if (byId('write-back').checked) { app.terminal().reset(); await write(app.terminal(), output); }
    app.refresh();
  });
  on('serialize-html', 'click', async () => {
    const output = requiredMethod(app.addons().ensure('serialize'), 'serializeAsHTML')();
    byId('html-output').textContent = output;
    await copyHtml(output);
    app.log('serialize:html-copied', output.length);
  });
  on('addon-load', 'click', () => app.addons().load(byId('addon-select').value, byId('socket-url').value));
  on('addon-unload', 'click', () => app.addons().unload(byId('addon-select').value));
  on('addon-reload', 'click', () => app.addons().reload(byId('addon-select').value, byId('socket-url').value));
  const observer = new ResizeObserver(() => { if (autoResize && app.isOpen()) app.action('auto-fit', fit); });
  observer.observe(byId('terminal-container'));
  return { rebuildOptions, applyTheme, dispose() { controller.abort(); optionController?.abort(); observer.disconnect(); } };
}
