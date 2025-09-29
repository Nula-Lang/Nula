#!/usr/bin/env bash
# open-nula.sh
# Uruchamia "nula help" w domyślnym terminalu (Linux / Unix / macOS).
# Zrób chmod +x open-nula.sh

CMD='nula help'

# Uruchom komendę w aktualnej terminalowej sesji (fallback dla serwera / ssh)
run_in_current_shell() {
  echo "Uruchamiam w bieżącej sesji: $CMD"
  bash -lc "$CMD"
  return $?
}

# Próbuje uruchomić komendę w podanym emulatorze terminala z argumentami -e lub -- / sh -c
try_terminal() {
  local term_cmd="$1"   # np. "gnome-terminal --"
  local exec_form="$2"  # forma wywołania: command_to_run w shellu
  if command -v "${term_cmd%% *}" >/dev/null 2>&1 || [[ -x "${term_cmd%% *}" ]]; then
    # Jeśli terminal to program GUI który akceptuje -e/-- bash -lc ... próbuj uruchomić
    eval "$exec_form"
    return $?
  fi
  return 1
}

uname_s="$(uname -s)"

case "$uname_s" in
  Darwin)
    # macOS: użyj AppleScript do otwarcia domyślnej aplikacji Terminal i wykonania komendy
    # Alternatywnie można dodać wsparcie dla iTerm — poniżej drobny tryb: najpierw iTerm, potem Terminal.
    if /usr/bin/osascript -e 'tell application "iTerm" to quit' >/dev/null 2>&1; then
      # nic — tylko test istnienia; ale wolimy sprawdzić obecność iTerm2 poprawniej:
      :
    fi

    # najpierw spróbuj iTerm2 (jeśli jest zainstalowany)
    if [ -d "/Applications/iTerm.app" ] || command -v /usr/local/bin/iterm2 >/dev/null 2>&1 || command -v iTerm >/dev/null 2>&1; then
      /usr/bin/osascript <<EOF
tell application "iTerm"
  activate
  try
    set newWindow to (create window with default profile)
    tell current session of newWindow to write text "$CMD"
  on error
    -- jeśli nie można utworzyć okna, spróbuj nowej sesji
    tell current session of (create window with default profile) to write text "$CMD"
  end try
end tell
EOF
      exit $?
    fi

    # fallback: macOS Terminal.app
    /usr/bin/osascript <<EOF
tell application "Terminal"
  activate
  do script "$CMD"
end tell
EOF
    exit $?
    ;;
  Linux|*BSD*|SunOS)
    # Lista popularnych emulatorów terminala i forma uruchomienia
    # Kolejność: użyj przypisanego $TERMINAL, następnie znane terminale.
    # Dla każdego terminala staramy się zostawić otwarte okno (exec bash na końcu)
    # oraz wykonać "nula help" w shellu loginowym (bash -lc).
    # Jeśli terminal obsługuje -- bash -lc '...; exec bash' -> użyj tego.
    # Tworzymy komendę do wywołania dynamicznie.

    # Jeżeli użytkownik podał $TERMINAL -> spróbuj go
    if [ -n "$TERMINAL" ]; then
      term_prog=$(echo "$TERMINAL" | awk '{print $1}')
      if command -v "$term_prog" >/dev/null 2>&1; then
        # spróbuj standardowego -e lub -- bash -lc
        if "$term_prog" --version >/dev/null 2>&1; then
          # wiele emulatorów obsługuje -- bash -lc
          eval "$term_prog -- bash -lc '$CMD; exec bash'" 2>/dev/null || \
          eval "$term_prog -e bash -lc '$CMD; exec bash'" 2>/dev/null
          exit $?
        fi
      fi
    fi

    # lista terminali (najpierw te graficzne)
    terminals=(
      "x-terminal-emulator"
      "gnome-terminal"
      "konsole"
      "xfce4-terminal"
      "mate-terminal"
      "lxterminal"
      "alacritty"
      "kitty"
      "terminator"
      "urxvt"
      "rxvt"
      "xterm"
    )

    for t in "${terminals[@]}"; do
      if command -v "$t" >/dev/null 2>&1; then
        case "$t" in
          gnome-terminal)
            gnome-terminal -- bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || true
            ;;
          konsole)
            konsole --hold -e bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || \
            konsole -e bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || true
            ;;
          xfce4-terminal)
            xfce4-terminal --command="bash -lc '$CMD; exec bash'" >/dev/null 2>&1 && exit 0 || true
            ;;
          mate-terminal|lxterminal|terminator)
            $t -e bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || true
            ;;
          alacritty|kitty)
            # alacritty/kitty używają -e bez dodatkowych flag
            $t -e bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || true
            ;;
          x-terminal-emulator)
            x-terminal-emulator -e bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || true
            ;;
          xterm|urxvt|rxvt)
            $t -hold -e bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || \
            $t -e bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || true
            ;;
          *)
            $t -e bash -lc "$CMD; exec bash" >/dev/null 2>&1 && exit 0 || true
            ;;
        esac
      fi
    done

    # jeśli doszliśmy tutaj — prawdopodobnie brak GUI terminala; uruchom w bieżącej sesji
    echo "Nie znaleziono graficznego emulatora terminala; uruchamiam w bieżącym shellu."
    run_in_current_shell
    exit $?
    ;;
  *)
    # nieznane/unix-like -> uruchom w bieżącym shellu
    echo "Niezidentyfikowany system ($uname_s). Uruchamiam w bieżącej sesji: $CMD"
    run_in_current_shell
    exit $?
    ;;
esac

