# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_qd_global_optspecs
    string join \n h/help V/version
end

function __fish_qd_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_qd_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_qd_using_subcommand
    set -l cmd (__fish_qd_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c qd -n "__fish_qd_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c qd -n "__fish_qd_needs_command" -s V -l version -d 'Print version'
complete -c qd -n "__fish_qd_needs_command" -f -a "status" -d 'Show current usage for every account this machine is signed in to'
complete -c qd -n "__fish_qd_needs_command" -f -a "watch" -d 'Keep showing usage, refreshing every minute. No provider is asked more often than its own limit, so most refreshes show cached numbers with their age'
complete -c qd -n "__fish_qd_needs_command" -f -a "daemon" -d 'Run the daemon until Ctrl-C: the one poller every consumer shares, with history and the local API. While it runs, `status` and `watch` ask it instead of polling'
complete -c qd -n "__fish_qd_needs_command" -f -a "mcp" -d 'Run the MCP server named quotadial on standard input and output, for agents such as Claude Code and Codex. Like `status`, it asks the daemon while one runs'
complete -c qd -n "__fish_qd_needs_command" -f -a "alerts" -d 'List the alert rules and what they told lately, or add, remove, or turn one on or off. Rules are written in plain words, such as `quotadial alerts add "warn me at 80%"`'
complete -c qd -n "__fish_qd_needs_command" -f -a "doctor" -d 'Print what support asks for, with nothing secret in it: the version, the system, the licence, the daemon, every account and where its sign-in is, the problems and the settings. The app\'s Send feedback attaches the same'
complete -c qd -n "__fish_qd_needs_command" -f -a "licence" -d 'Say whether this copy is QuotaDial Free or QuotaDial Pro, or add or remove its licence key. A key is checked on this machine, with no network call'
complete -c qd -n "__fish_qd_needs_command" -f -a "history" -d 'Show how each window was used over time, from the daemon\'s history'
complete -c qd -n "__fish_qd_needs_command" -f -a "claude" -d 'Commands the Claude provider adds'
complete -c qd -n "__fish_qd_needs_command" -f -a "antigravity" -d 'Commands the Antigravity provider adds'
complete -c qd -n "__fish_qd_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand status" -l json -d 'Print JSON instead of text'
complete -c qd -n "__fish_qd_using_subcommand status" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand watch" -l json -d 'Print one line of JSON per refresh instead of text'
complete -c qd -n "__fish_qd_using_subcommand watch" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand daemon" -l port -d 'The port for the local API, on 127.0.0.1. 0 lets the system choose a free one' -r
complete -c qd -n "__fish_qd_using_subcommand daemon" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand mcp" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand alerts; and not __fish_seen_subcommand_from list add remove on off help" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand alerts; and not __fish_seen_subcommand_from list add remove on off help" -f -a "list" -d 'List the rules and what they told in the last week. The default'
complete -c qd -n "__fish_qd_using_subcommand alerts; and not __fish_seen_subcommand_from list add remove on off help" -f -a "add" -d 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"'
complete -c qd -n "__fish_qd_using_subcommand alerts; and not __fish_seen_subcommand_from list add remove on off help" -f -a "remove" -d 'Remove the rule with this number'
complete -c qd -n "__fish_qd_using_subcommand alerts; and not __fish_seen_subcommand_from list add remove on off help" -f -a "on" -d 'Turn the rule with this number on'
complete -c qd -n "__fish_qd_using_subcommand alerts; and not __fish_seen_subcommand_from list add remove on off help" -f -a "off" -d 'Turn the rule with this number off'
complete -c qd -n "__fish_qd_using_subcommand alerts; and not __fish_seen_subcommand_from list add remove on off help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from list" -l json -d 'Print JSON instead of text'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from on" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from off" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from help" -f -a "list" -d 'List the rules and what they told in the last week. The default'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove the rule with this number'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from help" -f -a "on" -d 'Turn the rule with this number on'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from help" -f -a "off" -d 'Turn the rule with this number off'
complete -c qd -n "__fish_qd_using_subcommand alerts; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand doctor" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand licence; and not __fish_seen_subcommand_from show add remove help" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand licence; and not __fish_seen_subcommand_from show add remove help" -f -a "show" -d 'Say which QuotaDial this copy is, and from which licence. The default'
complete -c qd -n "__fish_qd_using_subcommand licence; and not __fish_seen_subcommand_from show add remove help" -f -a "add" -d 'Add a licence key for QuotaDial Pro, which replaces any kept before'
complete -c qd -n "__fish_qd_using_subcommand licence; and not __fish_seen_subcommand_from show add remove help" -f -a "remove" -d 'Remove the licence key, which makes this copy QuotaDial Free'
complete -c qd -n "__fish_qd_using_subcommand licence; and not __fish_seen_subcommand_from show add remove help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand licence; and __fish_seen_subcommand_from show" -l json -d 'Print JSON instead of text'
complete -c qd -n "__fish_qd_using_subcommand licence; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand licence; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand licence; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand licence; and __fish_seen_subcommand_from help" -f -a "show" -d 'Say which QuotaDial this copy is, and from which licence. The default'
complete -c qd -n "__fish_qd_using_subcommand licence; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add a licence key for QuotaDial Pro, which replaces any kept before'
complete -c qd -n "__fish_qd_using_subcommand licence; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove the licence key, which makes this copy QuotaDial Free'
complete -c qd -n "__fish_qd_using_subcommand licence; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand history" -l since -d 'From how long ago: such as 24h, 7d or 30d, or a time such as 2099-01-01T00:00:00Z' -r
complete -c qd -n "__fish_qd_using_subcommand history" -l provider -d 'Only this provider: claude, codex, gemini or antigravity' -r
complete -c qd -n "__fish_qd_using_subcommand history" -l account -d 'Only this account, by its email address or id' -r
complete -c qd -n "__fish_qd_using_subcommand history" -l window -d 'Only windows with this label, such as Session or Weekly' -r
complete -c qd -n "__fish_qd_using_subcommand history" -l csv -d 'Print every reading as CSV, for a spreadsheet'
complete -c qd -n "__fish_qd_using_subcommand history" -l json -d 'Print JSON, the same as `GET /v1/history`'
complete -c qd -n "__fish_qd_using_subcommand history" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand claude; and not __fish_seen_subcommand_from status-line help" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand claude; and not __fish_seen_subcommand_from status-line help" -f -a "status-line" -d 'Keeps the numbers Claude Code\'s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.'
complete -c qd -n "__fish_qd_using_subcommand claude; and not __fish_seen_subcommand_from status-line help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand claude; and __fish_seen_subcommand_from status-line" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand claude; and __fish_seen_subcommand_from help" -f -a "status-line" -d 'Keeps the numbers Claude Code\'s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.'
complete -c qd -n "__fish_qd_using_subcommand claude; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand antigravity; and not __fish_seen_subcommand_from status-line help" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand antigravity; and not __fish_seen_subcommand_from status-line help" -f -a "status-line" -d 'Keeps the numbers Antigravity CLI\'s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.'
complete -c qd -n "__fish_qd_using_subcommand antigravity; and not __fish_seen_subcommand_from status-line help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand antigravity; and __fish_seen_subcommand_from status-line" -s h -l help -d 'Print help'
complete -c qd -n "__fish_qd_using_subcommand antigravity; and __fish_seen_subcommand_from help" -f -a "status-line" -d 'Keeps the numbers Antigravity CLI\'s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.'
complete -c qd -n "__fish_qd_using_subcommand antigravity; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "status" -d 'Show current usage for every account this machine is signed in to'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "watch" -d 'Keep showing usage, refreshing every minute. No provider is asked more often than its own limit, so most refreshes show cached numbers with their age'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "daemon" -d 'Run the daemon until Ctrl-C: the one poller every consumer shares, with history and the local API. While it runs, `status` and `watch` ask it instead of polling'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "mcp" -d 'Run the MCP server named quotadial on standard input and output, for agents such as Claude Code and Codex. Like `status`, it asks the daemon while one runs'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "alerts" -d 'List the alert rules and what they told lately, or add, remove, or turn one on or off. Rules are written in plain words, such as `quotadial alerts add "warn me at 80%"`'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "doctor" -d 'Print what support asks for, with nothing secret in it: the version, the system, the licence, the daemon, every account and where its sign-in is, the problems and the settings. The app\'s Send feedback attaches the same'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "licence" -d 'Say whether this copy is QuotaDial Free or QuotaDial Pro, or add or remove its licence key. A key is checked on this machine, with no network call'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "history" -d 'Show how each window was used over time, from the daemon\'s history'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "claude" -d 'Commands the Claude provider adds'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "antigravity" -d 'Commands the Antigravity provider adds'
complete -c qd -n "__fish_qd_using_subcommand help; and not __fish_seen_subcommand_from status watch daemon mcp alerts doctor licence history claude antigravity help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from alerts" -f -a "list" -d 'List the rules and what they told in the last week. The default'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from alerts" -f -a "add" -d 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from alerts" -f -a "remove" -d 'Remove the rule with this number'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from alerts" -f -a "on" -d 'Turn the rule with this number on'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from alerts" -f -a "off" -d 'Turn the rule with this number off'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from licence" -f -a "show" -d 'Say which QuotaDial this copy is, and from which licence. The default'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from licence" -f -a "add" -d 'Add a licence key for QuotaDial Pro, which replaces any kept before'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from licence" -f -a "remove" -d 'Remove the licence key, which makes this copy QuotaDial Free'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from claude" -f -a "status-line" -d 'Keeps the numbers Claude Code\'s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.'
complete -c qd -n "__fish_qd_using_subcommand help; and __fish_seen_subcommand_from antigravity" -f -a "status-line" -d 'Keeps the numbers Antigravity CLI\'s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.'
