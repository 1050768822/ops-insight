const { invoke } = window.__TAURI__.core;
const { open: openDialog } = window.__TAURI__.dialog;

// ── Tab navigation ───────────────────────────────────────────────────────────

document.querySelectorAll('.tab').forEach(btn => {
  btn.addEventListener('click', () => switchTab(btn.dataset.tab));
});

function switchTab(name) {
  document.querySelectorAll('.tab').forEach(b =>
    b.classList.toggle('active', b.dataset.tab === name)
  );
  document.querySelectorAll('.panel').forEach(p =>
    p.classList.toggle('active', p.id === `tab-${name}`)
  );
  if (name === 'config') loadConfig();
}

// ── Report type radio ─────────────────────────────────────────────────────────

let serilogPath = '';

document.querySelectorAll('input[name=rtype]').forEach(r => {
  r.addEventListener('change', () => {
    const v = r.value;
    document.getElementById('custom-opts').classList.toggle('visible', v === 'custom');
    document.getElementById('serilog-opts').classList.toggle('visible', v === 'serilog');
  });
});

document.getElementById('pick-file-btn').addEventListener('click', async () => {
  const selected = await openDialog({ multiple: false, directory: false });
  if (selected) {
    serilogPath = selected;
    document.getElementById('serilog-path-label').textContent = selected;
  }
});

// ── Run report ───────────────────────────────────────────────────────────────

document.getElementById('run-btn').addEventListener('click', runReport);

async function runReport() {
  const rtype = document.querySelector('input[name=rtype]:checked').value;
  setProgress(true);
  setError('');

  try {
    let report;

    if (rtype === 'daily') {
      report = await invoke('generate_daily_report');

    } else if (rtype === 'weekly') {
      report = await invoke('generate_weekly_report');

    } else if (rtype === 'custom') {
      const from = document.getElementById('custom-from').value;
      const to   = document.getElementById('custom-to').value;
      if (!from || !to) throw new Error('请选择开始和结束日期');
      report = await invoke('generate_custom_report', { from, to });

    } else if (rtype === 'serilog') {
      if (!serilogPath) throw new Error('请先选择日志文件路径');
      const from = document.getElementById('serilog-from').value || null;
      const to   = document.getElementById('serilog-to').value   || null;
      report = await invoke('generate_serilog_report', {
        path: serilogPath,
        from,
        to,
      });
    }

    renderReport(report);
    switchTab('report');
  } catch (err) {
    setError(String(err));
  } finally {
    setProgress(false);
  }
}

// ── Render report ─────────────────────────────────────────────────────────────

function renderReport(report) {
  document.getElementById('report-empty').style.display   = 'none';
  document.getElementById('report-content').style.display = 'block';

  document.getElementById('report-title').textContent = report.title;

  const from = new Date(report.period.from).toLocaleDateString('zh-CN');
  const to   = new Date(report.period.to).toLocaleDateString('zh-CN');
  document.getElementById('report-period').textContent = `${from} — ${to}`;

  document.getElementById('report-summary').textContent = report.summary || '（无摘要）';

  // Issues
  const issuesList = document.getElementById('issues-list');
  issuesList.innerHTML = '';
  document.getElementById('issue-count').textContent = `(${report.issues.length})`;

  if (report.issues.length === 0) {
    issuesList.innerHTML = '<div class="empty-state" style="padding:20px 0">未发现问题</div>';
  } else {
    for (const issue of report.issues) {
      issuesList.appendChild(buildIssueCard(issue));
    }
  }

  // Suggestions
  const suggestionsList = document.getElementById('suggestions-list');
  suggestionsList.innerHTML = '';
  document.getElementById('suggestion-count').textContent = `(${report.suggestions.length})`;

  if (report.suggestions.length === 0) {
    suggestionsList.innerHTML = '<div class="empty-state" style="padding:20px 0">暂无建议</div>';
  } else {
    for (const s of report.suggestions) {
      suggestionsList.appendChild(buildSuggestionCard(s));
    }
  }
}

function buildIssueCard(issue) {
  const card = document.createElement('div');
  card.className = 'issue-card';

  const severity = issue.severity.toLowerCase();
  card.innerHTML = `
    <div class="issue-header">
      <span class="badge badge-${severity}">${severity}</span>
      <span class="issue-title">${escHtml(issue.title)}</span>
    </div>
    <div class="issue-desc">${escHtml(issue.description)}</div>
    <div class="issue-meta">
      <span>影响主机：${escHtml(issue.affected_hosts.join(', ') || '—')}</span>
      <span>发生次数：${issue.occurrence_count}</span>
    </div>
  `;
  return card;
}

function buildSuggestionCard(s) {
  const card = document.createElement('div');
  const priority = s.priority.toLowerCase();
  card.className = `suggestion-card priority-${priority}`;
  card.innerHTML = `
    <div class="suggestion-title">${escHtml(s.title)}</div>
    <div class="suggestion-detail">${escHtml(s.detail)}</div>
  `;
  return card;
}

// ── Config management ─────────────────────────────────────────────────────────

async function loadConfig() {
  try {
    const path    = await invoke('get_config_path');
    const content = await invoke('load_config_cmd');
    document.getElementById('config-path-display').textContent = path;
    document.getElementById('config-editor').value = content;
    setConfigMsg('');
  } catch (err) {
    document.getElementById('config-editor').value = '';
    setConfigMsg(`加载失败：${err}`);
  }
}

document.getElementById('save-config-btn').addEventListener('click', async () => {
  const content = document.getElementById('config-editor').value;
  try {
    await invoke('save_config_cmd', { content });
    setConfigMsg('保存成功');
  } catch (err) {
    setConfigMsg(`保存失败：${err}`);
  }
});

document.getElementById('init-config-btn').addEventListener('click', async () => {
  try {
    await invoke('init_config_cmd');
    setConfigMsg('配置模板已生成，请填写 API Key');
    await loadConfig();
  } catch (err) {
    setConfigMsg(`错误：${err}`);
  }
});

document.getElementById('reload-config-btn').addEventListener('click', loadConfig);

// ── Helpers ───────────────────────────────────────────────────────────────────

function setProgress(on) {
  document.getElementById('progress').classList.toggle('visible', on);
  document.getElementById('run-btn').disabled = on;
}

function setError(msg) {
  const el = document.getElementById('error-banner');
  el.textContent = msg;
  el.classList.toggle('visible', !!msg);
}

function setConfigMsg(msg) {
  document.getElementById('config-msg').textContent = msg;
}

function escHtml(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
