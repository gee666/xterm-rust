import { fullTraceScenarios } from '../../full-trace-scenarios.mjs';
import { runTrace } from '../../full-trace-runner.mjs';

export const caseIds = ['vt-23', 'vt-24', 'vt-44', 'vt-45', 'vt-46'];

export async function runCase(context) {
  if (!caseIds.includes(context.caseId)) throw new Error(`Unsupported print/control case ${context.caseId}`);
  return { caseId: context.caseId, ...await runTrace(context, fullTraceScenarios(context.caseId)) };
}
