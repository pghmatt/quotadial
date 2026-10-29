
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'quotadial' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'quotadial'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'quotadial' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current usage for every account this machine is signed in to')
            [CompletionResult]::new('watch', 'watch', [CompletionResultType]::ParameterValue, 'Keep showing usage, refreshing every minute. No provider is asked more often than its own limit, so most refreshes show cached numbers with their age')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Run the daemon until Ctrl-C: the one poller every consumer shares, with history and the local API. While it runs, `status` and `watch` ask it instead of polling')
            [CompletionResult]::new('mcp', 'mcp', [CompletionResultType]::ParameterValue, 'Run the MCP server named quotadial on standard input and output, for agents such as Claude Code and Codex. Like `status`, it asks the daemon while one runs')
            [CompletionResult]::new('alerts', 'alerts', [CompletionResultType]::ParameterValue, 'List the alert rules and what they told lately, or add, remove, or turn one on or off. Rules are written in plain words, such as `quotadial alerts add "warn me at 80%"`')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Print what support asks for, with nothing secret in it: the version, the system, the licence, the daemon, every account and what QuotaDial reads it from, the problems and the settings. The app''s Send feedback attaches the same')
            [CompletionResult]::new('licence', 'licence', [CompletionResultType]::ParameterValue, 'Say whether this copy is QuotaDial Free or QuotaDial Pro, or add or remove its licence key. A key is checked on this machine, with no network call')
            [CompletionResult]::new('history', 'history', [CompletionResultType]::ParameterValue, 'Show how each window was used over time, from the daemon''s history')
            [CompletionResult]::new('claude', 'claude', [CompletionResultType]::ParameterValue, 'Commands the Claude provider adds')
            [CompletionResult]::new('antigravity', 'antigravity', [CompletionResultType]::ParameterValue, 'Commands the Antigravity provider adds')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;status' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print JSON instead of text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;watch' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print one line of JSON per refresh instead of text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;daemon' {
            [CompletionResult]::new('--port', '--port', [CompletionResultType]::ParameterName, 'The port for the local API, on 127.0.0.1. 0 lets the system choose a free one')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;mcp' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;alerts' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List the rules and what they told in the last week. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the rule with this number')
            [CompletionResult]::new('on', 'on', [CompletionResultType]::ParameterValue, 'Turn the rule with this number on')
            [CompletionResult]::new('off', 'off', [CompletionResultType]::ParameterValue, 'Turn the rule with this number off')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;alerts;list' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print JSON instead of text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;alerts;add' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;alerts;remove' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;alerts;on' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;alerts;off' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;alerts;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List the rules and what they told in the last week. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the rule with this number')
            [CompletionResult]::new('on', 'on', [CompletionResultType]::ParameterValue, 'Turn the rule with this number on')
            [CompletionResult]::new('off', 'off', [CompletionResultType]::ParameterValue, 'Turn the rule with this number off')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;alerts;help;list' {
            break
        }
        'quotadial;alerts;help;add' {
            break
        }
        'quotadial;alerts;help;remove' {
            break
        }
        'quotadial;alerts;help;on' {
            break
        }
        'quotadial;alerts;help;off' {
            break
        }
        'quotadial;alerts;help;help' {
            break
        }
        'quotadial;doctor' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;licence' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Say which QuotaDial this copy is, and from which licence. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a licence key for QuotaDial Pro, which replaces any kept before')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the licence key, which makes this copy QuotaDial Free')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;licence;show' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print JSON instead of text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;licence;add' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;licence;remove' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;licence;help' {
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Say which QuotaDial this copy is, and from which licence. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a licence key for QuotaDial Pro, which replaces any kept before')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the licence key, which makes this copy QuotaDial Free')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;licence;help;show' {
            break
        }
        'quotadial;licence;help;add' {
            break
        }
        'quotadial;licence;help;remove' {
            break
        }
        'quotadial;licence;help;help' {
            break
        }
        'quotadial;history' {
            [CompletionResult]::new('--since', '--since', [CompletionResultType]::ParameterName, 'From how long ago: such as 24h, 7d or 30d, or a time such as 2099-01-01T00:00:00Z')
            [CompletionResult]::new('--provider', '--provider', [CompletionResultType]::ParameterName, 'Only this provider: claude, codex, gemini or antigravity')
            [CompletionResult]::new('--account', '--account', [CompletionResultType]::ParameterName, 'Only this account, by its email address or id')
            [CompletionResult]::new('--window', '--window', [CompletionResultType]::ParameterName, 'Only windows with this label, such as Session or Weekly')
            [CompletionResult]::new('--csv', '--csv', [CompletionResultType]::ParameterName, 'Print every reading as CSV, for a spreadsheet')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print JSON, the same as `GET /v1/history`')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;claude' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Claude Code''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;claude;status-line' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;claude;help' {
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Claude Code''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;claude;help;status-line' {
            break
        }
        'quotadial;claude;help;help' {
            break
        }
        'quotadial;antigravity' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Antigravity CLI''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;antigravity;status-line' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'quotadial;antigravity;help' {
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Antigravity CLI''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;antigravity;help;status-line' {
            break
        }
        'quotadial;antigravity;help;help' {
            break
        }
        'quotadial;help' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current usage for every account this machine is signed in to')
            [CompletionResult]::new('watch', 'watch', [CompletionResultType]::ParameterValue, 'Keep showing usage, refreshing every minute. No provider is asked more often than its own limit, so most refreshes show cached numbers with their age')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Run the daemon until Ctrl-C: the one poller every consumer shares, with history and the local API. While it runs, `status` and `watch` ask it instead of polling')
            [CompletionResult]::new('mcp', 'mcp', [CompletionResultType]::ParameterValue, 'Run the MCP server named quotadial on standard input and output, for agents such as Claude Code and Codex. Like `status`, it asks the daemon while one runs')
            [CompletionResult]::new('alerts', 'alerts', [CompletionResultType]::ParameterValue, 'List the alert rules and what they told lately, or add, remove, or turn one on or off. Rules are written in plain words, such as `quotadial alerts add "warn me at 80%"`')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Print what support asks for, with nothing secret in it: the version, the system, the licence, the daemon, every account and what QuotaDial reads it from, the problems and the settings. The app''s Send feedback attaches the same')
            [CompletionResult]::new('licence', 'licence', [CompletionResultType]::ParameterValue, 'Say whether this copy is QuotaDial Free or QuotaDial Pro, or add or remove its licence key. A key is checked on this machine, with no network call')
            [CompletionResult]::new('history', 'history', [CompletionResultType]::ParameterValue, 'Show how each window was used over time, from the daemon''s history')
            [CompletionResult]::new('claude', 'claude', [CompletionResultType]::ParameterValue, 'Commands the Claude provider adds')
            [CompletionResult]::new('antigravity', 'antigravity', [CompletionResultType]::ParameterValue, 'Commands the Antigravity provider adds')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'quotadial;help;status' {
            break
        }
        'quotadial;help;watch' {
            break
        }
        'quotadial;help;daemon' {
            break
        }
        'quotadial;help;mcp' {
            break
        }
        'quotadial;help;alerts' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List the rules and what they told in the last week. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the rule with this number')
            [CompletionResult]::new('on', 'on', [CompletionResultType]::ParameterValue, 'Turn the rule with this number on')
            [CompletionResult]::new('off', 'off', [CompletionResultType]::ParameterValue, 'Turn the rule with this number off')
            break
        }
        'quotadial;help;alerts;list' {
            break
        }
        'quotadial;help;alerts;add' {
            break
        }
        'quotadial;help;alerts;remove' {
            break
        }
        'quotadial;help;alerts;on' {
            break
        }
        'quotadial;help;alerts;off' {
            break
        }
        'quotadial;help;doctor' {
            break
        }
        'quotadial;help;licence' {
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Say which QuotaDial this copy is, and from which licence. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a licence key for QuotaDial Pro, which replaces any kept before')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the licence key, which makes this copy QuotaDial Free')
            break
        }
        'quotadial;help;licence;show' {
            break
        }
        'quotadial;help;licence;add' {
            break
        }
        'quotadial;help;licence;remove' {
            break
        }
        'quotadial;help;history' {
            break
        }
        'quotadial;help;claude' {
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Claude Code''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            break
        }
        'quotadial;help;claude;status-line' {
            break
        }
        'quotadial;help;antigravity' {
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Antigravity CLI''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            break
        }
        'quotadial;help;antigravity;status-line' {
            break
        }
        'quotadial;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
