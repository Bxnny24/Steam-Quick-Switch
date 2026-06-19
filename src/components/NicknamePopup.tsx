import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const params = new URLSearchParams(window.location.search);
const steamId64 = params.get("steamid") ?? "";
const accountName = params.get("name") ?? "";
const initialNick = params.get("nick") ?? "";
const lang = params.get("lang") === "de" ? "de" : "en";

const TEXT = {
  en: {
    title: "Nickname for",
    placeholder: "Leave empty to remove",
    save: "Save",
    cancel: "Cancel",
  },
  de: {
    title: "Spitzname für",
    placeholder: "Leer lassen zum Entfernen",
    save: "Speichern",
    cancel: "Abbrechen",
  },
}[lang];

export function NicknamePopup() {
  const [value, setValue] = useState(initialNick);

  function save() {
    invoke("set_nickname", { steamId64, nickname: value });
  }

  function cancel() {
    invoke("close_nickname");
  }

  return (
    <div className="popup">
      <div className="popup__label">
        {TEXT.title} <strong>{accountName}</strong>
      </div>
      <input
        className="popup__input"
        autoFocus
        type="text"
        placeholder={TEXT.placeholder}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") save();
          if (e.key === "Escape") cancel();
        }}
      />
      <div className="popup__actions">
        <button className="popup__btn" onClick={cancel}>
          {TEXT.cancel}
        </button>
        <button className="popup__btn popup__btn--primary" onClick={save}>
          {TEXT.save}
        </button>
      </div>
    </div>
  );
}
