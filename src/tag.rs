use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use crate::{branch, cat_file, config::Config, hash_object, utils};

/// List the tags in the specified directory and write their names to the given output.
///
/// # Arguments
///
/// * `tags_path` - A string slice representing the path to the directory containing tags.
/// * `output` - A mutable reference to an object implementing the `Write` trait where the
///   tag names will be written.
///
/// # Errors
///
/// If there is an error reading the directory or writing to the output, an `io::Result` is returned
/// with an appropriate error message. The caller should handle the result appropriately.
///
/// # Panics
///
/// This function panics if it encounters an error while writing the error message to the output.
fn list_tags(tags_path: &str, output: &mut impl Write) -> io::Result<()> {
    if let Ok(entries) = fs::read_dir(tags_path) {
        println!("Entries: {:?}", entries);
        for entry in entries.flatten() {
            println!("Entry {:?}", entry);
            let file_name = entry.file_name();
            if let Some(name) = file_name.to_str() {
                output.write_all(format!("{}\n", name).as_bytes())?;
            }
        }
    } else {
        output.write_all(
            format!("Error al abrir el directorio de tags: {}\n", tags_path).as_bytes(),
        )?;
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Error al abrir el directorio de tags: {}\n", tags_path),
        ));
    }
    Ok(())
}

/// Create a new Git tag with the specified name and associate it with the current commit.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git repository directory.
/// * `tags_path` - A string slice representing the path to the directory where tags are stored.
/// * `tag_name` - A string slice representing the name of the new tag to be created.
/// * `output` - A mutable reference to an object implementing the `Write` trait where
///   status messages or errors will be written.
///
/// # Errors
///
/// The function returns an `io::Result` indicating whether the tag creation was successful or
/// if there was an error during the process. Possible error scenarios include:
///
/// * The specified tag already exists, leading to a `AlreadyExists` error.
/// * Unable to retrieve the current commit, resulting in a `branch::get_current_branch_commit` error.
/// * Unable to create the new tag file or write the commit hash, leading to file-related errors.
///
/// # Panics
///
/// This function panics if it encounters an unexpected error while writing to the output.
fn create_tag(
    git_dir: &str,
    tags_path: &str,
    tag_name: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    let file_path = format!("{}/{}", tags_path, tag_name);
    let path = Path::new(&file_path);
    if path.exists() {
        output.write_all(format!("fatal: tag '{}' already exists\n", tag_name).as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("fatal: tag '{}' already exists", tag_name),
        ));
    }

    let commit = branch::get_current_branch_commit(git_dir)?;
    let mut new_file = File::create(&file_path)?;
    new_file.write_all(commit.as_bytes())?;
    new_file.flush()?;
    Ok(())
}

/// Create a new annotated Git tag with the specified name, message, and associate it with the current commit.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git repository directory.
/// * `tags_path` - A string slice representing the path to the directory where tags are stored.
/// * `tag_name` - A string slice representing the name of the new annotated tag to be created.
/// * `mensaje` - A string slice representing the message or annotation for the tag.
/// * `output` - A mutable reference to an object implementing the `Write` trait where
///   status messages or errors will be written.
///
/// # Errors
///
/// The function returns an `io::Result` indicating whether the annotated tag creation was successful or
/// if there was an error during the process. Possible error scenarios include:
///
/// * The specified tag already exists, leading to a `NotFound` error.
/// * Unable to retrieve the current commit hash, resulting in a `branch::get_current_branch_commit` error.
/// * Unable to get the timestamp or offset, resulting in a `utils::get_timestamp` error.
/// * Unable to load the Git configuration, resulting in a `Config::load` error.
/// * Unable to retrieve the user name and email from the configuration, leading to a `Config::get_user_name_and_email` error.
/// * Unable to create the new annotated tag file or write its content, leading to file-related errors.
/// * Unable to store the tag content as an object in the Git repository, leading to a `hash_object::store_string_to_file` error.
///
/// # Panics
///
/// This function panics if it encounters an unexpected error while writing to the output.
fn create_annotated_tag(
    git_dir: &str,
    tags_path: &str,
    tag_name: &str,
    mensaje: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    let file_path = format!("{}/{}", tags_path, tag_name);
    println!("File path {:?}", file_path);
    let path = Path::new(&file_path);
    if path.exists() {
        println!("Existe");
        output.write_all(format!("fatal: tag '{}' already exists\n", tag_name).as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("fatal: tag '{}' already exists", tag_name),
        ));
    }

    let (timestamp, offset) = utils::get_timestamp()?;
    println!("Timestamp {timestamp}, offset {offset}");
    let config = Config::load(git_dir)?;
    println!("Config loaded");
    let result = config.get_user_name_and_email();
    if result.is_err() {
        println!("Nombre y mail?");
        output.write_all(format!("{:?}", result.unwrap_err()).as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error con el config file.",
        ));
    }
    let (name, email) = result?;
    println!("Name {name}, email {email}");
    let commit = branch::get_current_branch_commit(git_dir)?;
    println!("Commit is {:?}", commit);
    let tag_content = format!(
        "object {}\ntype commit\ntag {}\ntagger {} {} {} {}\n\n{}\n",
        commit, tag_name, name, email, timestamp, offset, mensaje
    );
    let hash = hash_object::store_string_to_file(&tag_content, git_dir, "tag")?;
    println!("Hash is {:?}", hash);
    let mut new_file = File::create(&file_path)?;
    new_file.write_all(hash.as_bytes())?;
    new_file.flush()?;
    println!("File written");
    Ok(())
}

/// Copy an existing Git tag to create a new tag with a different name.
///
/// # Arguments
///
/// * `new_tag` - A string slice representing the name of the new tag to be created.
/// * `old_tag` - A string slice representing the name of the existing tag to be copied.
/// * `tags_path` - A string slice representing the path to the directory where tags are stored.
/// * `output` - A mutable reference to an object implementing the `Write` trait where
///   status messages or errors will be written.
///
/// # Errors
///
/// The function returns an `io::Result` indicating whether the tag copying was successful or
/// if there was an error during the process. Possible error scenarios include:
///
/// * The source tag does not exist, leading to a `fatal: Failed to resolve` error.
/// * The destination tag already exists, leading to a `fatal: tag already exists` error.
/// * Unable to read the content of the source tag file, resulting in a `fs::read_to_string` error.
/// * Unable to create the new tag file or write its content, leading to file-related errors.
///
/// # Panics
///
/// This function does not panic under normal circumstances. Panics may occur in case of unexpected errors
/// while writing to the output.
fn copy_tag(
    new_tag: &str,
    old_tag: &str,
    tags_path: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    let old_tag_path = format!("{}/{}", tags_path, old_tag);
    println!("Old tag path {old_tag_path}");
    let old_tag_path = Path::new(&old_tag_path);
    if !old_tag_path.exists() {
        output.write_all(
            format!("fatal: Failed to resolve '{}' as a valid ref.\n", old_tag).as_bytes(),
        )?;
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("fatal: Failed to resolve '{}' as a valid ref.\n", old_tag),
        ));
    }
    let new_tag_path = format!("{}/{}", tags_path, new_tag);
    println!("New tag path {new_tag_path}");
    let new_tag_path = Path::new(&new_tag_path);
    if new_tag_path.exists() {
        output.write_all(format!("fatal: tag '{}' already exists\n", new_tag).as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("fatal: tag '{}' already exists\n", new_tag),
        ));
    }
    let content = fs::read_to_string(old_tag_path)?;
    println!("content {content}");
    let mut new_tag_file = File::create(new_tag_path)?;
    new_tag_file.write_all(content.as_bytes())?;
    new_tag_file.flush()?;
    println!("Correctly written");
    Ok(())
}

/// Copy an existing Git tag to create a new tag with a different name.
///
/// # Arguments
///
/// * `new_tag` - A string slice representing the name of the new tag to be created.
/// * `old_tag` - A string slice representing the name of the existing tag to be copied.
/// * `tags_path` - A string slice representing the path to the directory where tags are stored.
/// * `output` - A mutable reference to an object implementing the `Write` trait where
///   status messages or errors will be written.
///
/// # Errors
///
/// The function returns an `io::Result` indicating whether the tag copying was successful or
/// if there was an error during the process. Possible error scenarios include:
///
/// * The source tag does not exist, leading to a `fatal: Failed to resolve` error.
/// * The destination tag already exists, leading to a `fatal: tag already exists` error.
/// * Unable to read the content of the source tag file, resulting in a `fs::read_to_string` error.
/// * Unable to create the new tag file or write its content, leading to file-related errors.
///
/// # Panics
///
/// This function does not panic under normal circumstances. Panics may occur in case of unexpected errors
/// while writing to the output.
fn delete_tag(tag_name: &str, tags_path: &str, output: &mut impl Write) -> io::Result<()> {
    let tag_to_delete_path = format!("{}/{}", tags_path, tag_name);
    println!("Tag to delete {:?}", tag_to_delete_path);
    let path = Path::new(&tag_to_delete_path);
    if !path.exists() {
        println!("No existe");
        output.write_all(format!("error: tag {} not found\n", tag_name).as_bytes())?;
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("error: tag {} not found\n", tag_name),
        ));
    } else {
        let content = fs::read_to_string(&tag_to_delete_path)?;
        let hash: String = content.chars().take(7).collect();
        fs::remove_file(&tag_to_delete_path)?;
        output.write_all(format!("Deleted tag '{}' (was {})\n", tag_name, hash).as_bytes())?;
    }

    Ok(())
}

/// Verify the integrity of an existing Git tag and display its information.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git repository directory.
/// * `tag_name` - A string slice representing the name of the tag to be verified.
/// * `tags_path` - A string slice representing the path to the directory where tags are stored.
/// * `output` - A mutable reference to an object implementing the `Write` trait where
///   tag information or errors will be written.
///
/// # Errors
///
/// The function returns an `io::Result` indicating whether the tag verification was successful or
/// if there was an error during the process. Possible error scenarios include:
///
/// * The specified tag does not exist, leading to a `error: tag not found` error.
/// * Unable to read the content of the tag file, resulting in a `fs::read_to_string` error.
/// * Unable to retrieve the object content using `cat_file::cat_file_return_content`, leading to an error.
/// * The object content is not a tag (starts with "tree"), leading to a `cannot verify a non-tag object of type commit` error.
///
/// # Panics
///
/// This function does not panic under normal circumstances. Panics may occur in case of unexpected errors
/// while writing to the output or processing the tag information.
fn verify_tag(
    git_dir: &str,
    tag_name: &str,
    tags_path: &str,
    output: &mut impl Write,
) -> io::Result<()> {
    let tag_to_verify_path = format!("{}/{}", tags_path, tag_name);
    let tag_to_verify_path = Path::new(&tag_to_verify_path);
    if !tag_to_verify_path.exists() {
        output.write_all(format!("error: tag '{}' not found.\n", tag_name).as_bytes())?;
    }
    let hash = fs::read_to_string(tag_to_verify_path)?;
    let result = cat_file::cat_file_return_content(&hash, git_dir);
    if result.is_err() {
        output.write_all(
            "error: couldn't cat_file the hash.\n"
                .to_string()
                .as_bytes(),
        )?;
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "error: couldn't cat_file the hash.\n".to_string(),
        ));
    } else {
        let content = result?;
        if content.starts_with("tree") {
            output.write_all(
                format!(
                    "error: {}: cannot verify a non-tag object of type commit.\n",
                    tag_name
                )
                .as_bytes(),
            )?;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "error: {}: cannot verify a non-tag object of type commit.\n",
                    tag_name
                ),
            ));
        } else {
            output.write_all(content.as_bytes())?;
        }
    }
    Ok(())
}

/// Interact with Git tags based on command line arguments.
///
/// # Arguments
///
/// * `git_dir` - A string slice representing the path to the Git repository directory.
/// * `line` - A vector of strings representing the command line arguments.
/// * `output` - A mutable reference to an object implementing the `Write` trait where
///   command results or errors will be written.
///
/// # Errors
///
/// The function returns an `io::Result` indicating whether the Git tag operation was successful or
/// if there was an error during the process. Specific errors are propagated from underlying tag-related functions.
///
/// # Panics
///
/// This function does not panic under normal circumstances. Panics may occur in case of unexpected errors
/// while writing to the output or when calling tag-related functions.
pub fn git_tag(git_dir: &str, line: Vec<String>, output: &mut impl Write) -> io::Result<()> {
    println!("Git dir is {:?}\n", git_dir);
    println!("Command received is {:?}\n", line);
    let tags_path = format!("{}/{}", git_dir, "refs/tags");
    println!("Tags path is {:?}\n", tags_path);
    if line.len() == 2 {
        println!("Listing tags!\n");
        list_tags(&tags_path, output)?;
    } else if line.len() == 3 {
        if line[2] == "-l" {
            println!("Listing tags!\n");
            list_tags(&tags_path, output)?;
        } else {
            println!("Creating tag...\n");
            create_tag(git_dir, &tags_path, &line[2], output)?;
        }
    } else if line.len() == 6 {
        if line[2] == "-a" {
            println!("Creating annotated tag...\n");
            create_annotated_tag(git_dir, &tags_path, &line[3], &line[5], output)?;
        }
    } else if line.len() >= 4 {
        if line[2] == "-d" {
            println!("Deleting tag...\n");
            let tags_to_delete: Vec<&String> = line.iter().skip(3).collect();
            for tag in tags_to_delete {
                delete_tag(tag, &tags_path, output)?;
            }
        } else if line[2] == "-v" {
            println!("Verifying tag...\n");
            let tags_to_verify: Vec<&String> = line.iter().skip(3).collect();
            for tag in tags_to_verify {
                verify_tag(git_dir, tag, &tags_path, output)?;
            }
        } else {
            println!("Copying tag");
            copy_tag(&line[2], &line[3], &tags_path, output)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::{commit, init};

    use super::*;

    fn create_if_not_exists(path: &str, is_dir: bool) -> io::Result<()> {
        if !Path::new(path).exists() {
            if is_dir {
                std::fs::create_dir(path)?;
            } else {
                File::create(path)?;
            }
        }
        Ok(())
    }

    fn create_repo(path: &str) -> Result<(), io::Error> {
        create_if_not_exists(path, true)?;
        init::git_init(path, "current_branch", None)?;
        let git_dir_path = format!("{}/{}", path, ".mgit");
        let git_ignore_path = format!("{}/{}", path, ".mgitignore");
        let tags_path = format!("{}/{}", git_dir_path, "refs/tags");
        create_if_not_exists(&tags_path, true)?;
        create_if_not_exists(&git_dir_path, false)?;
        commit::new_commit(
            &git_dir_path,
            "This is a commit for tag test.",
            &git_ignore_path,
        )?;

        Ok(())
    }

    #[test]
    fn create_non_existing_tag_succeeds() -> io::Result<()> {
        let path = "tests/tag_fake_repo_01";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = create_tag(&git_dir, &tags_path, "v2", &mut output);
        assert!(result.is_ok());
        let new_tag_path = format!("{}/{}", tags_path, "v2");
        let new_tag_path = Path::new(&new_tag_path);
        assert!(new_tag_path.exists());
        let current_commit = branch::get_current_branch_commit(&git_dir)?;
        let new_tag_commit = fs::read_to_string(new_tag_path)?;
        assert!(current_commit.eq(&new_tag_commit));
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn create_existing_tag_fails() -> io::Result<()> {
        let path = "tests/tag_fake_repo_02";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = create_tag(&git_dir, &tags_path, "v2", &mut output);
        assert!(result.is_ok());
        let result = create_tag(&git_dir, &tags_path, "v2", &mut output);
        assert!(result.is_err());
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn create_non_existing_annotated_tag_succeeds() -> io::Result<()> {
        let path = "tests/tag_fake_repo_03";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let config_file_path = format!("{}/{}", &git_dir, "config");
        let content = fs::read_to_string(&config_file_path)?;
        let content = content + "[user]\n\tname = Claris\n\temail = crfrugoli@unmail.com.ar\n";
        let mut config_file = File::create(&config_file_path)?;
        config_file.write_all(content.as_bytes())?;
        let mut output: Vec<u8> = vec![];
        let result = create_annotated_tag(
            &git_dir,
            &tags_path,
            "v2",
            "Create annotated tag test.",
            &mut output,
        );
        assert!(result.is_ok());
        let new_tag_path = format!("{}/{}", tags_path, "v2");
        let new_tag_path = Path::new(&new_tag_path);
        assert!(new_tag_path.exists());
        let current_commit = branch::get_current_branch_commit(&git_dir)?;
        let new_tag_commit = fs::read_to_string(new_tag_path)?;
        assert!(!current_commit.eq(&new_tag_commit));
        let new_tag_content = cat_file::cat_file_return_content(&new_tag_commit, &git_dir)?;
        assert!(new_tag_content.contains(&current_commit));
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn create_existing_annotated_tag_fails() -> io::Result<()> {
        let path = "tests/tag_fake_repo_04";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let config_file_path = format!("{}/{}", &git_dir, "config");
        let content = fs::read_to_string(&config_file_path)?;
        let content = content + "[user]\n\tname = Claris\n\temail = crfrugoli@unmail.com.ar\n";
        let mut config_file = File::create(&config_file_path)?;
        config_file.write_all(content.as_bytes())?;
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = create_annotated_tag(
            &git_dir,
            &tags_path,
            "v2",
            "Create annotated tag test.",
            &mut output,
        );
        assert!(result.is_ok());
        let result = create_annotated_tag(
            &git_dir,
            &tags_path,
            "v2",
            "Create annotated tag test.",
            &mut output,
        );
        assert!(result.is_err());
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn create_non_existing_annotated_tag_without_user_info_fails() -> io::Result<()> {
        let path = "tests/tag_fake_repo_05";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = create_annotated_tag(
            &git_dir,
            &tags_path,
            "v2",
            "Create annotated tag test.",
            &mut output,
        );
        assert!(result.is_err());
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn list_tags_lists_correctly() -> io::Result<()> {
        let path = "tests/tag_fake_repo_06";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        create_tag(&git_dir, &tags_path, "v2", &mut output)?;
        create_tag(&git_dir, &tags_path, "v3", &mut output)?;
        create_tag(&git_dir, &tags_path, "v4", &mut output)?;
        list_tags(&tags_path, &mut output)?;
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("v2"));
        assert!(output_string.contains("v3"));
        assert!(output_string.contains("v4"));
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn list_tags_fails_if_tags_directory_does_not_exist() -> io::Result<()> {
        let path = "tests/tag_fake_repo_07";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        fs::remove_dir_all(&tags_path)?;
        let mut output: Vec<u8> = vec![];
        let result = list_tags(&tags_path, &mut output);
        assert!(result.is_err());
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn delete_non_existing_tag_throws_error() -> io::Result<()> {
        let path = "tests/tag_fake_repo_08";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = delete_tag("v2", &tags_path, &mut output);
        assert!(result.is_err());
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn delete_existing_tag_returns_ok() -> io::Result<()> {
        let path = "tests/tag_fake_repo_09";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = create_tag(&git_dir, &tags_path, "v2", &mut output);
        assert!(result.is_ok());
        let tag_path = format!("{}/{}", tags_path, "v2");
        let tag_path = Path::new(&tag_path);
        assert!(tag_path.exists());
        let result = delete_tag("v2", &tags_path, &mut output);
        assert!(result.is_ok());
        assert!(!tag_path.exists());
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn verify_non_annotated_tag_returns_error() -> io::Result<()> {
        let path = "tests/tag_fake_repo_10";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = create_tag(&git_dir, &tags_path, "v2", &mut output);
        assert!(result.is_ok());
        let result = verify_tag(&git_dir, "v2", &tags_path, &mut output);
        assert!(result.is_err());
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn verify_annotated_tag_prints_tag_content() -> io::Result<()> {
        let path = "tests/tag_fake_repo_11";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let config_file_path = format!("{}/{}", &git_dir, "config");
        let content = fs::read_to_string(&config_file_path)?;
        let content = content + "[user]\n\tname = Claris\n\temail = crfrugoli@unmail.com.ar\n";
        let mut config_file = File::create(&config_file_path)?;
        config_file.write_all(content.as_bytes())?;
        let mut output: Vec<u8> = vec![];
        let result = create_annotated_tag(
            &git_dir,
            &tags_path,
            "v2",
            "Create annotated tag test.",
            &mut output,
        );
        assert!(result.is_ok());
        let result = verify_tag(&git_dir, "v2", &tags_path, &mut output);
        assert!(result.is_ok());
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.starts_with("object"));
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn copy_non_existing_tag_from_existing_one_returns_ok() -> io::Result<()> {
        let path = "tests/tag_fake_repo_12";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = create_tag(&git_dir, &tags_path, "v2", &mut output);
        assert!(result.is_ok());
        let result = copy_tag("v3", "v2", &tags_path, &mut output);
        assert!(result.is_ok());
        let new_tag_path = format!("{}/{}", tags_path, "v3");
        let new_tag_path = Path::new(&new_tag_path);
        let old_tag_path = format!("{}/{}", tags_path, "v2");
        let old_tag_path = Path::new(&old_tag_path);
        assert!(new_tag_path.exists());
        assert!(old_tag_path.exists());
        let new_tag_content = fs::read_to_string(new_tag_path)?;
        let old_tag_content = fs::read_to_string(old_tag_path)?;
        assert!(new_tag_content.eq(&old_tag_content));
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn copy_existing_tag_from_existing_one_returns_error() -> io::Result<()> {
        let path = "tests/tag_fake_repo_13";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = create_tag(&git_dir, &tags_path, "v2", &mut output);
        assert!(result.is_ok());
        let result = create_tag(&git_dir, &tags_path, "v3", &mut output);
        assert!(result.is_ok());
        let result = copy_tag("v3", "v2", &tags_path, &mut output);
        assert!(result.is_err());
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("already exists"));
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn copy_non_existing_tag_from_non_existing_one_returns_error() -> io::Result<()> {
        let path = "tests/tag_fake_repo_14";
        create_repo(path)?;
        let git_dir = format!("{}/{}", path, ".mgit");
        let tags_path = format!("{}/{}", git_dir, "refs/tags");
        let mut output: Vec<u8> = vec![];
        let result = copy_tag("v3", "v2", &tags_path, &mut output);
        assert!(result.is_err());
        let output_string = String::from_utf8(output).unwrap();
        assert!(output_string.contains("as a valid ref"));
        fs::remove_dir_all(path)?;
        Ok(())
    }
}
