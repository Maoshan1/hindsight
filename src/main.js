import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const queryInput = document.getElementById('query');
const resultsDiv = document.getElementById('results');
const resultSummary = document.getElementById('result-summary');
const refreshButton = document.getElementById('refresh');
const clearButton = document.getElementById('clear-query');
const toast = document.getElementById('copy-toast');
const filterButtons = [...document.querySelectorAll('[data-filter]')];
const openSettingsButton = document.getElementById('open-settings');
const settingsBackdrop = document.getElementById('settings-backdrop');
const closeSettingsButton = document.getElementById('close-settings');
const cancelSettingsButton = document.getElementById('cancel-settings');
const saveSettingsButton = document.getElementById('save-settings');
const recordingToggle = document.getElementById('recording-toggle');
const retentionSelect = document.getElementById('retention-select');
const clearHistoryButton = document.getElementById('clear-history');
const clearConfirmation = document.getElementById('clear-confirmation');
const cancelClearButton = document.getElementById('cancel-clear');
const confirmClearButton = document.getElementById('confirm-clear');
const settingsStatus = document.getElementById('settings-status');

const state = {
  query: '',
  items: [],
  filter: 'all',
  selectedIndex: 0,
  loading: false,
  error: '',
};

let debounceTimer = null;
let toastTimer = null;
let searchSequence = 0;

function escapeHtml(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function displayText(value) {
  return String(value ?? '').replace(/\s+/g, ' ').trim();
}

function displayPath(path) {
  const value = displayText(path);
  return value.replace(/^\/Users\/[^/]+(?=\/|$)/, '~');
}

function formatTimestamp(timestamp) {
  const date = new Date(Number(timestamp) * 1000);
  if (Number.isNaN(date.getTime())) return 'Unknown time';

  const now = new Date();
  const sameDay = date.toDateString() === now.toDateString();
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);

  const time = new Intl.DateTimeFormat(undefined, {
    hour: 'numeric',
    minute: '2-digit',
  }).format(date);

  if (sameDay) return `Today, ${time}`;
  if (date.toDateString() === yesterday.toDateString()) return `Yesterday, ${time}`;

  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  }).format(date);
}

function formatDuration(duration) {
  const milliseconds = Number(duration);
  if (!Number.isFinite(milliseconds) || milliseconds <= 0) return '';
  if (milliseconds < 1000) return `${Math.round(milliseconds)}ms`;

  const seconds = milliseconds / 1000;
  return `${seconds >= 10 ? seconds.toFixed(0) : seconds.toFixed(1).replace(/\.0$/, '')}s`;
}

function getStatus(item) {
  if (item.exit_code === 0) return { className: 'success', label: 'Succeeded' };
  if (item.exit_code === null || item.exit_code === undefined) {
    return { className: 'unknown', label: 'No exit status' };
  }
  return { className: 'failed', label: `Exited with ${item.exit_code}` };
}

function normalizeItem(item) {
  return {
    id: item.id,
    timestamp: item.timestamp,
    command: item.command ?? '',
    cwd: item.cwd ?? '',
    exit_code: item.exit_code,
    duration: item.duration,
  };
}

function visibleItems() {
  return state.items
    .map((item, index) => ({ item, index }))
    .filter(({ item }) => state.filter === 'all' || (
      item.exit_code !== null
      && item.exit_code !== undefined
      && item.exit_code !== 0
    ));
}

function renderToolbar() {
  filterButtons.forEach((button) => {
    button.classList.toggle('active', button.dataset.filter === state.filter);
  });

  if (state.loading) {
    resultSummary.textContent = 'Loading';
    return;
  }

  const count = visibleItems().length;
  resultSummary.textContent = `${count} ${count === 1 ? 'command' : 'commands'}`;
}

function renderLoading() {
  resultsDiv.innerHTML = Array.from({ length: 6 }, () => `
    <div class="skeleton-item" aria-hidden="true">
      <span class="skeleton-dot"></span>
      <span class="skeleton-copy">
        <span class="skeleton-command"></span>
        <span class="skeleton-meta"></span>
      </span>
    </div>
  `).join('');
}

function renderEmpty() {
  const heading = state.query ? 'No matching commands' : 'No commands recorded yet';
  const detail = state.query
    ? 'Try a different command or folder.'
    : 'Commands will appear here after your shell hook is active.';

  resultsDiv.innerHTML = `
    <div class="empty-state">
      <div class="empty-icon" aria-hidden="true">⌕</div>
      <h2>${heading}</h2>
      <p>${detail}</p>
    </div>
  `;
}

function renderError() {
  resultsDiv.innerHTML = `
    <div class="empty-state error-state">
      <div class="empty-icon" aria-hidden="true">!</div>
      <h2>History is unavailable</h2>
      <p>${escapeHtml(state.error)}</p>
      <button class="retry-button" type="button" data-action="retry">Try again</button>
    </div>
  `;
}

function renderResults() {
  if (state.loading) {
    renderLoading();
    return;
  }

  if (state.error) {
    renderError();
    return;
  }

  const entries = visibleItems();
  if (entries.length === 0) {
    renderEmpty();
    return;
  }

  state.selectedIndex = Math.min(state.selectedIndex, entries.length - 1);
  resultsDiv.innerHTML = entries.map(({ item, index }, entryIndex) => {
    const status = getStatus(item);
    const command = displayText(item.command) || '(empty command)';
    const cwd = displayPath(item.cwd) || 'Unknown folder';
    const duration = formatDuration(item.duration);
    const exitCode = status.className === 'failed'
      ? `<span class="meta-divider" aria-hidden="true">·</span><span class="exit-code">exit ${escapeHtml(item.exit_code)}</span>`
      : '';

    return `
      <button
        class="history-item ${entryIndex === state.selectedIndex ? 'selected' : ''}"
        type="button"
        role="option"
        aria-selected="${entryIndex === state.selectedIndex}"
        aria-label="Copy command: ${escapeHtml(command)}"
        data-index="${index}"
      >
        <span class="status-dot ${status.className}" title="${escapeHtml(status.label)}"></span>
        <span class="history-copy">
          <span class="command" title="${escapeHtml(item.command)}">${escapeHtml(command)}</span>
          <span class="meta">
            <span>${escapeHtml(formatTimestamp(item.timestamp))}</span>
            <span class="meta-divider" aria-hidden="true">·</span>
            <span class="cwd" title="${escapeHtml(item.cwd)}">${escapeHtml(cwd)}</span>
            ${duration ? `<span class="meta-divider" aria-hidden="true">·</span><span>${escapeHtml(duration)}</span>` : ''}
            ${exitCode}
          </span>
        </span>
        <span class="copy-mark" aria-hidden="true">⧉</span>
      </button>
    `;
  }).join('');

  const selected = resultsDiv.querySelector('.history-item.selected');
  selected?.scrollIntoView({ block: 'nearest' });
}

function render() {
  renderToolbar();
  renderResults();
  clearButton.classList.toggle('visible', state.query.length > 0);
}

function formatError(error) {
  const message = String(error ?? 'Unknown error').replace(/^Error:\s*/i, '');
  return message.length > 160 ? `${message.slice(0, 157)}...` : message;
}

function setSettingsStatus(message, isError = false) {
  settingsStatus.textContent = message;
  settingsStatus.classList.toggle('error', isError);
}

function setSettingsForm(settings) {
  recordingToggle.checked = settings.recording_enabled !== false;
  retentionSelect.value = settings.retention_days == null ? '' : String(settings.retention_days);
}

function resetClearConfirmation() {
  clearConfirmation.hidden = true;
  clearHistoryButton.hidden = false;
  confirmClearButton.disabled = false;
}

function closeSettings() {
  settingsBackdrop.hidden = true;
  resetClearConfirmation();
  setSettingsStatus('');
  queryInput.focus();
}

async function openSettings() {
  resetClearConfirmation();
  settingsBackdrop.hidden = false;
  setSettingsStatus('Loading settings...');
  saveSettingsButton.disabled = true;
  clearHistoryButton.disabled = true;
  closeSettingsButton.focus();

  try {
    const settings = await invoke('get_settings');
    setSettingsForm(settings);
    setSettingsStatus('');
  } catch (error) {
    setSettingsStatus(formatError(error), true);
  } finally {
    saveSettingsButton.disabled = false;
    clearHistoryButton.disabled = false;
  }
}

async function saveSettings() {
  saveSettingsButton.disabled = true;
  setSettingsStatus('Saving...');

  try {
    await invoke('update_settings', {
      settings: {
        recording_enabled: recordingToggle.checked,
        retention_days: retentionSelect.value ? Number(retentionSelect.value) : null,
      },
    });
    closeSettings();
    showToast('Settings saved');
    search(state.query);
  } catch (error) {
    setSettingsStatus(formatError(error), true);
  } finally {
    saveSettingsButton.disabled = false;
  }
}

function requestClearHistory() {
  clearHistoryButton.hidden = true;
  clearConfirmation.hidden = false;
  setSettingsStatus('Review the confirmation below.');
  cancelClearButton.focus();
}

async function clearHistory() {
  confirmClearButton.disabled = true;
  clearHistoryButton.disabled = true;
  setSettingsStatus('Clearing history...');

  try {
    const removed = await invoke('clear_history');
    // Ignore a search request that started before the delete completed.
    searchSequence += 1;
    state.items = [];
    state.selectedIndex = 0;
    state.loading = false;
    state.error = '';
    resetClearConfirmation();
    render();
    setSettingsStatus(`${removed} ${removed === 1 ? 'record' : 'records'} removed.`);
    showToast(`${removed} ${removed === 1 ? 'record' : 'records'} cleared`);
  } catch (error) {
    setSettingsStatus(formatError(error), true);
  } finally {
    confirmClearButton.disabled = false;
    clearHistoryButton.disabled = false;
  }
}

async function search(query = state.query) {
  state.query = query;
  const sequence = ++searchSequence;
  state.loading = true;
  state.error = '';
  render();

  try {
    const results = await invoke('search_commands', { query: state.query, limit: 100 });
    if (sequence !== searchSequence) return;
    state.items = Array.isArray(results) ? results.map(normalizeItem) : [];
    state.selectedIndex = 0;
  } catch (error) {
    if (sequence !== searchSequence) return;
    state.items = [];
    state.error = formatError(error);
  } finally {
    if (sequence === searchSequence) {
      state.loading = false;
      render();
    }
  }
}

function showToast(message, isError = false) {
  clearTimeout(toastTimer);
  toast.textContent = message;
  toast.classList.toggle('error', isError);
  toast.classList.add('show');
  toastTimer = setTimeout(() => toast.classList.remove('show'), 1800);
}

async function copyToClipboard(text) {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    } else {
      const textarea = document.createElement('textarea');
      textarea.value = text;
      textarea.setAttribute('readonly', '');
      textarea.style.position = 'fixed';
      textarea.style.opacity = '0';
      document.body.appendChild(textarea);
      textarea.select();
      const copied = document.execCommand('copy');
      textarea.remove();
      if (!copied) throw new Error('Clipboard access was denied');
    }
    showToast('Command copied');
  } catch (error) {
    showToast(formatError(error), true);
  }
}

function copySelected() {
  const entries = visibleItems();
  const selected = entries[state.selectedIndex];
  if (selected) copyToClipboard(selected.item.command);
}

queryInput.addEventListener('input', () => {
  state.query = queryInput.value;
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => search(state.query), 100);
});

queryInput.addEventListener('keydown', (event) => {
  const entries = visibleItems();

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      if (entries.length > 0) state.selectedIndex = Math.min(state.selectedIndex + 1, entries.length - 1);
      render();
      break;
    case 'ArrowUp':
      event.preventDefault();
      state.selectedIndex = Math.max(state.selectedIndex - 1, 0);
      render();
      break;
    case 'Enter':
      event.preventDefault();
      copySelected();
      break;
    case 'Escape':
      if (state.query) {
        event.preventDefault();
        queryInput.value = '';
        search('');
      }
      break;
  }
});

document.addEventListener('keydown', (event) => {
  if (!settingsBackdrop.hidden) {
    if (event.key === 'Escape') {
      event.preventDefault();
      closeSettings();
    }
    return;
  }

  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault();
    queryInput.focus();
    queryInput.select();
  }
});

resultsDiv.addEventListener('click', (event) => {
  const item = event.target.closest('.history-item');
  if (!item) {
    if (event.target.closest('[data-action="retry"]')) search(state.query);
    return;
  }

  const entries = visibleItems();
  const originalIndex = Number(item.dataset.index);
  const entryIndex = entries.findIndex(({ index }) => index === originalIndex);
  if (entryIndex >= 0) state.selectedIndex = entryIndex;

  const selected = state.items[originalIndex];
  if (selected) copyToClipboard(selected.command);
  render();
});

filterButtons.forEach((button) => {
  button.addEventListener('click', () => {
    state.filter = button.dataset.filter;
    state.selectedIndex = 0;
    render();
  });
});

refreshButton.addEventListener('click', () => search(state.query));

openSettingsButton.addEventListener('click', openSettings);
closeSettingsButton.addEventListener('click', closeSettings);
cancelSettingsButton.addEventListener('click', closeSettings);
saveSettingsButton.addEventListener('click', saveSettings);
clearHistoryButton.addEventListener('click', requestClearHistory);
cancelClearButton.addEventListener('click', () => {
  resetClearConfirmation();
  setSettingsStatus('');
  clearHistoryButton.focus();
});
confirmClearButton.addEventListener('click', clearHistory);

settingsBackdrop.addEventListener('click', (event) => {
  if (event.target === settingsBackdrop) closeSettings();
});

clearButton.addEventListener('click', () => {
  queryInput.value = '';
  queryInput.focus();
  search('');
});

listen('window-shown', () => {
  queryInput.focus();
  search(state.query);
}).catch(() => {
  // The listener is unavailable when the frontend is opened outside Tauri.
});

search('');
