import type { GameConfig, GameMeta, GameOption } from "../engine/types";
import { optionLabel, optionSummary, valueLabel } from "../games/options";

interface Props {
  meta: GameMeta;
  config: GameConfig | null;
  /** Absent: the options are only shown (guest, table). */
  onChange?(config: GameConfig): void;
}

/** A game's options, from what the engine announces: editable in the lobby, a summary elsewhere. */
export function GameOptions({ meta, config, onChange }: Props) {
  if (meta.options.length === 0 || config === null) return null;
  if (!onChange) {
    return (
      <ul className="options-summary">
        {optionSummary(meta, config).map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
    );
  }
  const set = (key: string, value: boolean | string | number) => onChange({ ...config, [key]: value });
  return (
    <div className="options">
      {meta.options.map((option) => (
        <div className="field" key={option.key}>
          <label htmlFor={`option-${option.key}`}>{optionLabel(meta.id, option.key)}</label>
          <OptionInput gameId={meta.id} option={option} value={config[option.key]} set={(v) => set(option.key, v)} />
        </div>
      ))}
    </div>
  );
}

function OptionInput({
  gameId,
  option,
  value,
  set,
}: {
  gameId: string;
  option: GameOption;
  value: boolean | string | number | undefined;
  set(value: boolean | string | number): void;
}) {
  const id = `option-${option.key}`;
  const kind = option.kind;
  switch (kind.type) {
    case "toggle":
      return <input id={id} type="checkbox" checked={value === true} onChange={(e) => set(e.target.checked)} />;
    case "choice":
      return (
        <select id={id} value={String(value)} onChange={(e) => set(e.target.value)}>
          {kind.values.map((v) => (
            <option key={v} value={v}>
              {valueLabel(gameId, option.key, v)}
            </option>
          ))}
        </select>
      );
    case "number":
      // Every value in bounds is listed: nothing to validate here.
      return (
        <select id={id} value={String(value)} onChange={(e) => set(Number(e.target.value))}>
          {Array.from({ length: kind.max - kind.min + 1 }, (_, k) => kind.min + k).map((n) => (
            <option key={n} value={n}>
              {n}
            </option>
          ))}
        </select>
      );
  }
}
