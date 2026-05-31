import { invoke } from '@tauri-apps/api/core';

const queryInput = document.getElementById('query');
const resultsDiv = document.getElementById('results');
let selectedIndex = 0;
let items = [];

async function search(query) {
  try {
    const results = await invoke('search_commands', { query, limit: 50 });
    items = results.map(r => {
      const parts = r.split('\t');
      return {
        time: parts[0] || '',
        exit: parts[1] || '?',
        cmd: parts[2] || '',
        cwd: parts[3] || ''
      };
    });
    render();
  } catch (e) {
    resultsDiv.innerHTML = `<div class="empty">Error: ${String(e)}</div>`;
  }
}

function render() {
  if (items.length === 0) {
    resultsDiv.innerHTML = '<div class="empty">No results</div>';
    return;
  }
  resultsDiv.innerHTML = items.map((item, i) => {
    const exitClass = item.exit === '✓' ? 'ok' : 'fail';
    return `<div class="item ${i === selectedIndex ? 'selected' : ''}" data-index="${i}">
      <span class="time">${item.time}</span>
      <span class="exit ${exitClass}">${item.exit}</span>
      <span class="cmd" title="${escapeHtml(item.cmd)}">${escapeHtml(item.cmd)}</span>
      <span class="cwd" title="${escapeHtml(item.cwd)}">${escapeHtml(item.cwd)}</span>
    </div>`;
  }).join('');

  const selected = resultsDiv.querySelector('.item.selected');
  if (selected) selected.scrollIntoView({ block: 'nearest' });
}

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function copyToClipboard(text) {
  navigator.clipboard.writeText(text);
  let toast = document.querySelector('.copy-toast');
  if (!toast) {
    toast = document.createElement('div');
    toast.className = 'copy-toast';
    document.body.appendChild(toast);
  }
  toast.textContent = `Copied: ${text}`;
  toast.classList.add('show');
  setTimeout(() => toast.classList.remove('show'), 1500);
}

let debounceTimer = null;
queryInput.addEventListener('input', () => {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => search(queryInput.value), 80);
});

queryInput.addEventListener('keydown', (e) => {
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
      render();
      break;
    case 'ArrowUp':
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      render();
      break;
    case 'Enter':
      e.preventDefault();
      if (items[selectedIndex]) copyToClipboard(items[selectedIndex].cmd);
      break;
  }
});

resultsDiv.addEventListener('click', (e) => {
  const item = e.target.closest('.item');
  if (item) {
    const index = parseInt(item.dataset.index);
    if (items[index]) copyToClipboard(items[index].cmd);
  }
});

// Refresh data when search input gets focus (window shown)
queryInput.addEventListener('focus', () => {
  search(queryInput.value);
});

search('');
