import { Group, Loader, Progress, Text } from "@mantine/core";

export function TaskProgress({ label, percent }: { label: string; percent: number | null }) {
  return <div className="task-progress">
    {percent !== null && <Progress value={percent} aria-label={label} />}
    <Group gap={8} wrap="nowrap">
      {percent === null && <Loader size={16} aria-hidden />}
      <Text size="sm" role="status" aria-live="polite">{label}</Text>
      {percent !== null && <Text size="sm" aria-hidden>{percent}%</Text>}
    </Group>
  </div>;
}
