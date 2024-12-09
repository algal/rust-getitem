use crate::pick;
use crate::slice_from_spec;
use crate::normalize_indices;

// Helper functions equivalent to Python's f and g
fn strip_first_newline(s: &str) -> String {
    if s.starts_with('\n') {
        s[1..].to_string()
    } else {
        s.to_string()
    }
}

fn prepare_test_case(
    input: &str,
    row_spec: &str,
    col_spec: &str,
    expected: &str,
) -> (String, (String, String), String) {
    (
        strip_first_newline(input),
        (row_spec.to_string(), col_spec.to_string()),
        strip_first_newline(expected),
    )
}

#[test]
fn test_case_s0() {
    let input = r#"
On branch dev-longcontexteval
Your branch is up to date with 'origin/dev-longcontexteval'.

Untracked files:
  (use "git add <file>..." to include in what will be committed)
    bert24-base-v2.yaml
    r_first50000.json
    src/evals/items2000.json
    src/evals/rewritten10.json

nothing added to commit but untracked files present (use "git add" to track)
"#;

    let expected = r#"
    bert24-base-v2.yaml
    r_first50000.json
    src/evals/items2000.json
    src/evals/rewritten10.json
"#;

    let (input, (row_spec, col_spec), expected) =
        prepare_test_case(input, "5:-2", ":", expected);

    let lines: Vec<String> = input.lines().map(String::from).collect();

    // Pass the length of the lines to `pick`
    let line_count = Some(lines.len());

    let actual: Vec<String> = pick(lines.clone(), &row_spec, &col_spec, line_count).collect();

    let expected: Vec<String> = expected.lines().map(String::from).collect();

    assert_eq!(actual, expected);
}

#[test]
fn test_case_s1() {
    let input = r#"
AAA
 BBB
CCC
"#;

    let expected = r#"
 BBB
"#;

    let (input, (row_spec, col_spec), expected) = prepare_test_case(input, "1", ":", expected);

    let lines: Vec<String> = input.lines().map(String::from).collect();

    let line_count = Some(lines.len());

    let actual: Vec<String> = pick(lines.clone(), &row_spec, &col_spec, line_count).collect();

    let expected: Vec<String> = expected.lines().map(String::from).collect();

    assert_eq!(actual, expected);
}

#[test]
fn test_case_s2() {
    let dirstring = r#"
.rw-r--r-- 0 root      2024-11-14 20:59 .localized
drwxr-x--- - alexis    2024-11-25 16:29 alexis
drwxr-xr-x - oldalexis 2023-09-17 14:13 alexis_1
drwxrwxrwt - root      2024-11-21 12:25 Shared
"#;

    let expected = dirstring;

    let (input, (row_spec, col_spec), expected) = prepare_test_case(dirstring, ":", ":", expected);

    let lines: Vec<String> = input.lines().map(String::from).collect();

    let line_count = Some(lines.len());

    let actual: Vec<String> = pick(lines.clone(), &row_spec, &col_spec, line_count).collect();

    let expected: Vec<String> = expected.lines().map(String::from).collect();

    assert_eq!(actual, expected);
}

#[test]
fn test_case_s3() {
    let dirstring = r#"
.rw-r--r-- 0 root      2024-11-14 20:59 .localized
drwxr-x--- - alexis    2024-11-25 16:29 alexis
drwxr-xr-x - oldalexis 2023-09-17 14:13 alexis_1
drwxrwxrwt - root      2024-11-21 12:25 Shared
"#;

    let expected = r#"
.rw-r--r-- 0 root      2024-11-14 20:59 .localized
drwxr-x--- - alexis    2024-11-25 16:29 alexis
"#;

    let (input, (row_spec, col_spec), expected) =
        prepare_test_case(dirstring, "0:2", ":", expected);

    let lines: Vec<String> = input.lines().map(String::from).collect();

    let line_count = Some(lines.len());

    let actual: Vec<String> = pick(lines.clone(), &row_spec, &col_spec, line_count).collect();

    let expected: Vec<String> = expected.lines().map(String::from).collect();

    assert_eq!(actual, expected);
}

#[test]
fn test_case_s4() {
    let dirstring = r#"
.rw-r--r-- 0 root      2024-11-14 20:59 .localized
drwxr-x--- - alexis    2024-11-25 16:29 alexis
drwxr-xr-x - oldalexis 2023-09-17 14:13 alexis_1
drwxrwxrwt - root      2024-11-21 12:25 Shared
"#;

    let expected = r#"
drwxr-xr-x - oldalexis 2023-09-17 14:13 alexis_1
drwxrwxrwt - root      2024-11-21 12:25 Shared
"#;

    let (input, (row_spec, col_spec), expected) =
        prepare_test_case(dirstring, "-2:", ":", expected);

    let lines: Vec<String> = input.lines().map(String::from).collect();

    let line_count = Some(lines.len());

    let actual: Vec<String> = pick(lines.clone(), &row_spec, &col_spec, line_count).collect();

    let expected: Vec<String> = expected.lines().map(String::from).collect();

    assert_eq!(actual, expected);
}

#[test]
fn test_case_s5() {
    let dirstring = r#"
.rw-r--r-- 0 root      2024-11-14 20:59 .localized
drwxr-x--- - alexis    2024-11-25 16:29 alexis
drwxr-xr-x - oldalexis 2023-09-17 14:13 alexis_1
drwxrwxrwt - root      2024-11-21 12:25 Shared
"#;

    let expected = r#"
                       2023-09-17 14:13
"#;

    let (input, (row_spec, col_spec), expected) =
        prepare_test_case(dirstring, "-2", "-3:-1", expected);

    let lines: Vec<String> = input.lines().map(String::from).collect();

    let line_count = Some(lines.len());

    let actual: Vec<String> = pick(lines.clone(), &row_spec, &col_spec, line_count).collect();

    let expected: Vec<String> = expected.lines().map(String::from).collect();

    assert_eq!(actual, expected);
}

#[test]
fn test_case_s6() {
    let input = r#"
#!/bin/bash
if [ "$#" -ne 2 ]; then
    echo "usage: make-linkfile.bash HTTP-URL TITLE"
    echo
    echo "To find links to nightlies, go to https://github.com/tensorflow/swift/blob/master/Installation.md "
    echo
    echo "A valid URL will look something like: https://storage.googleapis.com/swift-tensorflow-artifacts/releases/v0.3/rc1/swift-tensorflow-RELEASE-0.3-cuda10.0-cudnn7-ubuntu18.04.tar.gz"
    exit 1
else
    url="$1"
    title="$2"
fi

cat <<EOF > "${title}.html"
<!DOCTYPE html><html>
  <head><title>Redirecting to: ${url}</title>
  <meta http-equiv = "refresh" content = "0;url='${url}'" />
</head></html>
EOF

echo "Created file $(readlink -f "${title}.html")"
"#;

    let expected = r#"
if [ "$#" -ne 2 ]; then
    echo "usage: make-linkfile.bash HTTP-URL TITLE"
    echo
    echo "To find links to nightlies, go to https://github.com/tensorflow/swift/blob/master/Installation.md "
    echo
    echo "A valid URL will look something like: https://storage.googleapis.com/swift-tensorflow-artifacts/releases/v0.3/rc1/swift-tensorflow-RELEASE-0.3-cuda10.0-cudnn7-ubuntu18.04.tar.gz"
    exit 1
else
    url="$1"
"#;

    let (input, (row_spec, col_spec), expected) =
        prepare_test_case(input, "1:10", ":", expected);

    let lines: Vec<String> = input.lines().map(String::from).collect();

    let line_count = Some(lines.len());

    let actual: Vec<String> = pick(lines.clone(), &row_spec, &col_spec, line_count).collect();

    let expected: Vec<String> = expected.lines().map(String::from).collect();

    assert_eq!(actual, expected);
}
