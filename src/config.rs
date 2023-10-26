use crate::{remote_handler::Remote, branch_handler::Branch};
use std::{collections::LinkedList, fs::{File, OpenOptions}, io::{BufReader, BufRead, self, Write}};

#[derive(Default)]
pub struct Config {
    config_file_path : String,
    remotes : Vec<Remote>,
    branches : Vec<Branch>,
}

impl Config {
    // Constructor
    fn new(config_file_path : String) -> Config {
        let config = Config {
            config_file_path,
            remotes: Vec::new(),
            branches: Vec::new(),
        };
        config
    }

    pub fn load(git_dir: &str) -> io::Result<Config> {
        let file_name = format!("{}/config", git_dir);
        let mut config = Config::new((&file_name).to_string());
        let file = File::open(&file_name)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines().skip(5);
        let mut buffer: Vec<String> = Vec::new();
        let mut count = 0;

        loop {
            if count == 3 {
                for line in &buffer {
                    if line.starts_with("[remote") {
                        let splitted_name : Vec<&str> = (&buffer[0]).split('"').collect();
                        let name = (&splitted_name[1]).to_string();
                        let splitted_url : Vec<&str> = (&buffer[1]).split(' ').collect();
                        let url = (&splitted_url[2]).to_string();
                        let splitted_fetch : Vec<&str> = (&buffer[2]).split(' ').collect();
                        let fetch = (&splitted_fetch[2]).to_string();
                        //println!("name {} url {} fetch {}",name,url,fetch);
                        let remote = Remote::new(name,url,fetch);
                        config.remotes.push(remote);
                    }
                    else if line.starts_with("[branch") {
                        let splitted_name : Vec<&str> = (&buffer[0]).split('"').collect();
                        let name = (&splitted_name[1]).to_string();
                        let splitted_remote : Vec<&str> = (&buffer[1]).split(' ').collect();
                        let remote = (&splitted_remote[2]).to_string();
                        let splitted_merge : Vec<&str> = (&buffer[2]).split(' ').collect();
                        let merge = (&splitted_merge[2]).to_string();
                        //println!("name {} remote {} merge {}",name,remote,merge);
                        let branch = Branch::new(name,remote,merge);
                        config.branches.push(branch);
                    }
                }
                buffer.clear();
                count = 0;
            }

            match lines.next() {
                Some(Ok(line)) => {
                    buffer.push(line);
                    count += 1;
                }
                Some(Err(err)) => {
                    eprintln!("Error al leer línea: {}", err);
                }
                None => {
                    break;
                }
            }
        }
        Ok(config)
    }

    pub fn add_remote(&mut self, name : String, url : String, fetch : String) -> io::Result<()> {
        let remote = Remote::new((&name).to_string(),(&url).to_string(),(&fetch).to_string());
        self.remotes.push(remote);
        let mut file = OpenOptions::new().append(true).open(&self.config_file_path)?;
        let data_to_append = format!("[remote {}]\n\turl = {}\n\tfetch = {}\n", name, url, fetch);
        file.write_all(data_to_append.as_bytes())?;
        file.flush()?;
        Ok(())

    }

    pub fn add_branch(&mut self, name : String, remote : String, merge : String) -> io::Result<()> {
        let branch = Branch::new((&name).to_string(),(&remote).to_string(),(&merge).to_string());
        self.branches.push(branch);
        let mut file = OpenOptions::new().append(true).open(&self.config_file_path)?;
        let data_to_append = format!("[branch {}]\n\tremote = {}\n\tmerge = {}\n", name, remote, merge);
        file.write_all(data_to_append.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    pub fn remove_from_file(&mut self, name: &str, type_ : &str) -> io::Result<()> {
        let input_file = File::open(&self.config_file_path)?;
        let reader = BufReader::new(input_file);

        let temp_file_path = (&self.config_file_path).to_string() + "2";
        let output_file = File::create(&temp_file_path)?;
        let mut writer = io::BufWriter::new(output_file);

        let mut skip_lines = 0;

        for line in reader.lines() {
            let line = line?;
            if line.starts_with(&format!("[{} \"{}\"]", type_, name)) {
                skip_lines = 3; // Si la línea coincide, establece el contador a 3 para omitir las siguientes dos líneas.
            } else if skip_lines > 0 {
                skip_lines -= 1; // Omitir la línea actual si el contador es mayor que 0.
            } else {
                writeln!(writer, "{}", line)?;
            }
        }

        std::fs::rename(temp_file_path, &self.config_file_path)?;

        Ok(())
    }

    pub fn remove_remote(&mut self, name: &str) -> io::Result<()> {
        if let Some(index) = self.remotes.iter().position(|r| r.name == name) {
            self.remotes.remove(index);
            self.remove_from_file(name,"remote")?;
        } else {
            eprintln!("error: No such remote: '{}'", name);
            //return error
        }
        Ok(())
    }

    pub fn remove_branch(&mut self, name: &str) -> io::Result<()> {
        if let Some(index) = self.branches.iter().position(|b| b.name == name) {
            self.branches.remove(index);
            self.remove_from_file(name,"branch")?;
        } else {
            //Personalizar el mensaje de error o el error en sí
            eprintln!("error: No such branch: '{}'", name);
            //return error
        }
        Ok(())
    }
    
      



    

}
