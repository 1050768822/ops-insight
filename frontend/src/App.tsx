import { useState } from 'react'
import { GenerateTab } from './components/GenerateTab'
import { ReportTab } from './components/ReportTab'
import { ConfigTab } from './components/ConfigTab'
import { DesensitizeTab } from './components/DesensitizeTab'
import type { ReportDto } from './types'
import styles from './App.module.css'

type Tab = 'generate' | 'report' | 'config' | 'desensitize'

const TAB_LABELS: Record<Tab, string> = {
  generate:    '生成报告',
  report:      '报告详情',
  config:      '配置管理',
  desensitize: '脱敏配置',
}

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('generate')
  const [report, setReport] = useState<ReportDto | null>(null)

  const handleReportReady = (r: ReportDto) => {
    setReport(r)
    setActiveTab('report')
  }

  return (
    <div className={styles.layout}>
      <nav className={styles.nav}>
        {(['generate', 'report', 'config', 'desensitize'] as Tab[]).map(tab => (
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
        {activeTab === 'report'      && <ReportTab report={report} />}
        {activeTab === 'config'      && <ConfigTab />}
        {activeTab === 'desensitize' && <DesensitizeTab />}
      </main>
    </div>
  )
}
