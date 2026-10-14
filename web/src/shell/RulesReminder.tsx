/** A game's rules in a few lines, folded by default. Nothing without rules. */
export function RulesReminder({ rules, open }: { rules: string[] | undefined; open?: boolean }) {
  if (!rules || rules.length === 0) return null;
  return (
    <details className="rules" open={open}>
      <summary>Rappel des règles</summary>
      <ul>
        {rules.map((rule) => (
          <li key={rule}>{rule}</li>
        ))}
      </ul>
    </details>
  );
}
