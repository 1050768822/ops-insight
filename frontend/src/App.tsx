import { useState } from 'react'
import { GenerateTab } from './components/GenerateTab'
import { ReportTab } from './components/ReportTab'
import { ConfigTab } from './components/ConfigTab'
import { DesensitizeTab } from './components/DesensitizeTab'
import { PromptTab } from './components/PromptTab'
import type { GenerateReportsResultDto } from './types'
import styles from './App.module.css'

type Tab = 'generate' | 'report' | 'prompt' | 'config' | 'desensitize'

const TAB_LABELS: Record<Tab, string> = {
  generate:    '生成报告',
  report:      '报告详情',
  prompt:      'Prompt 配置',
  config:      '配置管理',
  desensitize: '脱敏配置',
}

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('generate')
  const [reportResult, setReportResult] = useState<GenerateReportsResultDto | null>(null)

  const handleReportReady = (result: GenerateReportsResultDto) => {
    setReportResult(result)
    setActiveTab('report')
  }

  const latestReport = reportResult?.reports[0]?.report ?? null
  const totalIssues = latestReport?.issues.length ?? 0
  const totalSuggestions = latestReport?.suggestions.length ?? 0
  const analyzerCount = reportResult?.reports.length ?? 0

  return (
    <div className={styles.layout}>
      <div className={styles.backdrop} />
      <header className={styles.hero}>
        <div className={styles.heroCopy}>
          <div className={styles.eyebrow}>OPS INSIGHT CONTROL</div>
          <h1 className={styles.title}>运维日志分析工作台</h1>
          <p className={styles.subtitle}>
            在一处完成日志采集、规则检测、Prompt 调整和历史报告回看。
          </p>
        </div>
        <div className={styles.metrics}>
          <div className={styles.metricCard}>
            <span className={styles.metricLabel}>ACTIVE ANALYZERS</span>
            <strong className={styles.metricValue}>{analyzerCount || '0'}</strong>
          </div>
          <div className={styles.metricCard}>
            <span className={styles.metricLabel}>LATEST ISSUES</span>
            <strong className={styles.metricValue}>{totalIssues}</strong>
          </div>
          <div className={styles.metricCard}>
            <span className={styles.metricLabel}>SUGGESTIONS</span>
            <strong className={styles.metricValue}>{totalSuggestions}</strong>
          </div>
        </div>
      </header>

      <div className={styles.shell}>
        <nav className={styles.nav}>
          {(['generate', 'report', 'prompt', 'config', 'desensitize'] as Tab[]).map(tab => (
            <button
              key={tab}
              className={`${styles.tab} ${activeTab === tab ? styles.active : ''}`}
              onClick={() => setActiveTab(tab)}
            >
              <span className={styles.tabLabel}>{TAB_LABELS[tab]}</span>
              <span className={styles.tabHint}>
                {tab === 'generate' && 'Run'}
                {tab === 'report' && 'Review'}
                {tab === 'prompt' && 'Tune'}
                {tab === 'config' && 'Manage'}
                {tab === 'desensitize' && 'Protect'}
              </span>
            </button>
          ))}
        </nav>

        <main className={styles.main}>
          {activeTab === 'generate'    && <GenerateTab onReportReady={handleReportReady} />}
          {activeTab === 'report'      && <ReportTab result={reportResult} />}
          {activeTab === 'prompt'      && <PromptTab />}
          {activeTab === 'config'      && <ConfigTab />}
          {activeTab === 'desensitize' && <DesensitizeTab />}
        </main>
      </div>
    </div>
  )
}
