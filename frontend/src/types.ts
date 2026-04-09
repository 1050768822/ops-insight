export interface PatternConfig {
  name: string;
  pattern: string;
  enabled: boolean;
}

export interface DesensitizeConfig {
  enabled: boolean;
  disabled_builtin: string[];
  custom_patterns: PatternConfig[];
}

export interface ReportPeriod {
  from: string;
  to: string;
}

export interface IssueDto {
  severity: 'critical' | 'high' | 'medium' | 'low';
  title: string;
  description: string;
  affected_hosts: string[];
  occurrence_count: number;
}

export interface SuggestionDto {
  priority: string;
  title: string;
  detail: string;
}

export interface ReportDto {
  title: string;
  period: ReportPeriod;
  summary: string;
  issues: IssueDto[];
  suggestions: SuggestionDto[];
}

export type AnalyzerId = 'local' | 'claude' | 'openai' | 'deepseek';

export interface GeneratedReportDto {
  analyzer: AnalyzerId;
  report: ReportDto;
}

export interface AnalyzerFailureDto {
  analyzer: AnalyzerId;
  reason: string;
}

export interface GenerateReportsResultDto {
  reports: GeneratedReportDto[];
  failures: AnalyzerFailureDto[];
  outputDir: string;
}

export interface ReportHistoryItemDto {
  fileName: string;
  path: string;
  modifiedAt?: number;
}

export interface ReportHistoryContentDto {
  fileName: string;
  path: string;
  content: string;
}

export interface AnalyzerOptionsDto {
  analyzers: AnalyzerOptionDto[];
}

export interface PromptConfigDto {
  zh: string;
  en: string;
}

export interface AnalyzerOptionDto {
  id: AnalyzerId;
  enabled: boolean;
  reason?: string;
  selectedByDefault: boolean;
}
