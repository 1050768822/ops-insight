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

  return (
    <div className={styles.layout}>
      <nav className={styles.nav}>
        {(['generate', 'report', 'prompt', 'config', 'desensitize'] as Tab[]).map(tab => (
          <button
            key={tab}
            className={`${styles.tab} ${activeTab === tab ? styles.active : ''}`}
            onClick={() => setActiveTab(tab)}
          >
            {TAB_LABELS[tab]}
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
  )
}
