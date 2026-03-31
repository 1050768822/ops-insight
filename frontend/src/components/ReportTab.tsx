import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { AnalyzerId, GenerateReportsResultDto, IssueDto, ReportDto, SuggestionDto } from '../types'
import styles from './ReportTab.module.css'

interface Props {
  result: GenerateReportsResultDto | null
}

const ANALYZER_LABELS: Record<AnalyzerId, string> = {
  local: '本地规则',
  claude: 'Claude',
  openai: 'OpenAI',
  deepseek: 'DeepSeek',
}

const SEVERITY_LABEL: Record<IssueDto['severity'], string> = {
  critical: '危急',
  high:     '高',
  medium:   '中',
  low:      '低',
}

function IssueCard({ issue }: { issue: IssueDto }) {
  return (
    <div className={styles.card}>
      <div className={styles.issueHeader}>
        <span className={`${styles.badge} ${styles[`sev_${issue.severity}`]}`}>
          {SEVERITY_LABEL[issue.severity]}
        </span>
        <span className={styles.issueTitle}>{issue.title}</span>
      </div>
      <p className={styles.issueDesc}>{issue.description}</p>
      <div className={styles.issueMeta}>
        <span>影响主机：{issue.affected_hosts.join(', ') || '—'}</span>
        <span>发生次数：{issue.occurrence_count}</span>
      </div>
    </div>
  )
}

function SuggestionCard({ s }: { s: SuggestionDto }) {
  const prio = s.priority.toLowerCase()
  return (
    <div className={`${styles.card} ${styles[`prio_${prio}`]}`}>
      <div className={styles.suggTitle}>{s.title}</div>
      <div className={styles.suggDetail}>{s.detail}</div>
    </div>
  )
}

export function ReportTab({ result }: Props) {
  const [activeAnalyzer, setActiveAnalyzer] = useState<AnalyzerId | null>(null)
  const [folderMessage, setFolderMessage] = useState('')
  const [folderMessageType, setFolderMessageType] = useState<'ok' | 'err'>('ok')

  useEffect(() => {
    setActiveAnalyzer(result?.reports[0]?.analyzer ?? null)
    setFolderMessage('')
  }, [result])

  if (!result || result.reports.length === 0) {
    return (
      <div className={styles.empty}>
        还没有报告，请先在「生成报告」页面运行分析。
      </div>
    )
  }

  const current = result.reports.find(item => item.analyzer === activeAnalyzer) ?? result.reports[0]
  const report: ReportDto = current.report
  const from = new Date(report.period.from).toLocaleDateString('zh-CN')
  const to   = new Date(report.period.to).toLocaleDateString('zh-CN')

  const openFolder = async () => {
    try {
      await invoke('open_report_folder', { path: result.outputDir })
      setFolderMessage(`已打开目录：${result.outputDir}`)
      setFolderMessageType('ok')
    } catch (e) {
      setFolderMessage(String(e))
      setFolderMessageType('err')
    }
  }

  return (
    <div className={styles.wrap}>
      <div className={styles.header}>
        <h1 className={styles.title}>{report.title}</h1>
        <div className={styles.period}>{from} — {to}</div>
        <div className={styles.toolbar}>
          <div className={styles.analyzerRow}>
            {result.reports.map(item => (
              <button
                key={item.analyzer}
                className={`${styles.analyzerChip} ${item.analyzer === current.analyzer ? styles.analyzerChipActive : ''}`}
                onClick={() => setActiveAnalyzer(item.analyzer)}
              >
                {ANALYZER_LABELS[item.analyzer]}
              </button>
            ))}
          </div>
          <button className={styles.openBtn} onClick={openFolder}>打开所在文件夹</button>
        </div>
        {report.summary && <div className={styles.summary}>{report.summary}</div>}
        {folderMessage && (
          <div className={folderMessageType === 'ok' ? styles.msgOk : styles.msgErr}>{folderMessage}</div>
        )}
      </div>

      <div className={styles.sectionTitle}>
        发现问题 <span className={styles.count}>({report.issues.length})</span>
      </div>
      {report.issues.length === 0
        ? <div className={styles.emptySection}>未发现问题</div>
        : report.issues.map((issue, i) => <IssueCard key={i} issue={issue} />)
      }

      <div className={styles.sectionTitle}>
        优化建议 <span className={styles.count}>({report.suggestions.length})</span>
      </div>
      {report.suggestions.length === 0
        ? <div className={styles.emptySection}>暂无建议</div>
        : report.suggestions.map((s, i) => <SuggestionCard key={i} s={s} />)
      }
    </div>
  )
}
