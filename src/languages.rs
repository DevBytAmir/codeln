//! Extension-to-language display name lookup, used by `counter` to group
//! files into the language breakdown.

/// Map a file extension to a display language name, falling back to the
/// extension itself when it isn't in the known table.
pub fn get_language_name(ext: &str) -> &str {
    match ext.to_lowercase().as_str() {
        // Web
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS/Sass",
        "less" => "Less",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" => "TypeScript",
        "jsx" => "React JSX",
        "tsx" => "React TSX",
        "vue" => "Vue",
        "svelte" => "Svelte",

        // Systems
        "rs" => "Rust",
        "c" => "C",
        "h" => "C Header",
        "cpp" | "cc" | "cxx" => "C++",
        "hpp" | "hxx" | "hh" => "C++ Header",
        "go" => "Go",
        "zig" => "Zig",

        // Scripting
        "py" | "pyw" => "Python",
        "rb" => "Ruby",
        "php" => "PHP",
        "pl" | "pm" => "Perl",
        "lua" => "Lua",
        "sh" | "bash" | "zsh" => "Shell",

        // JVM
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "scala" => "Scala",
        "groovy" => "Groovy",

        // .NET
        "cs" => "C#",
        "fs" | "fsi" | "fsx" => "F#",
        "vb" => "Visual Basic",

        // Functional
        "hs" | "lhs" => "Haskell",
        "ml" | "mli" => "OCaml",
        "clj" | "cljs" | "cljc" => "Clojure",
        "lisp" | "lsp" => "Lisp",
        "scm" | "ss" => "Scheme",
        "erl" | "hrl" => "Erlang",
        "ex" | "exs" => "Elixir",

        // Mobile
        "swift" => "Swift",
        "m" => "Objective-C",
        "dart" => "Dart",

        // Data/Config
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "sql" => "SQL",
        "proto" => "Protocol Buffers",

        // Markup/Docs
        "md" | "markdown" => "Markdown",
        "rst" => "reStructuredText",
        "tex" => "LaTeX",
        "adoc" | "asciidoc" => "AsciiDoc",

        // Other
        "r" => "R",
        "jl" => "Julia",
        "nim" => "Nim",
        "v" => "V",
        "d" => "D",
        "cr" => "Crystal",

        // Default
        _ => ext,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_known_extensions_case_insensitively() {
        assert_eq!(get_language_name("rs"), "Rust");
        assert_eq!(get_language_name("RS"), "Rust");
        assert_eq!(get_language_name("py"), "Python");
        assert_eq!(get_language_name("tsx"), "React TSX");
    }

    #[test]
    fn falls_back_to_the_extension_itself_when_unknown() {
        assert_eq!(get_language_name("xyz"), "xyz");
    }
}
