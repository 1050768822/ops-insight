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
