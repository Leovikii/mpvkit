import type { Asset, Report } from './types';

export function enhancementAction(report: Report | null, assets: Asset[]) {
  if (!report) return null;
  const components = report.components.filter(component => component.id !== 'mpv');
  if (!components.some(component => component.version)) return 'installOnly';
  if (assets.some(asset => !components.find(component => component.id === asset.id)?.ready)) return 'repairOnly';
  if (assets.some(asset => components.find(component => component.id === asset.id)?.version !== asset.version)) return 'updateComponents';
  return null;
}
