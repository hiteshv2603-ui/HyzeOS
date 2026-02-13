const MAX_FILES: usize = 32;
const MAX_FILENAME_LEN: usize = 32;
const MAX_FILE_SIZE: usize = 4096;
const MAX_PATH_LEN: usize = 128;
const MAX_DIRS: usize = 16;

#[derive(Clone, Copy)]
pub struct File {
    pub name: [u8; MAX_FILENAME_LEN],
    pub name_len: usize,
    pub data: [u8; MAX_FILE_SIZE],
    pub size: usize,
    pub is_used: bool,
    pub parent_dir: usize,
}

impl File {
    pub const fn empty() -> Self {
        Self {
            name: [0; MAX_FILENAME_LEN],
            name_len: 0,
            data: [0; MAX_FILE_SIZE],
            size: 0,
            is_used: false,
            parent_dir: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Directory {
    pub name: [u8; MAX_FILENAME_LEN],
    pub name_len: usize,
    pub is_used: bool,
    pub parent_dir: usize,
}

impl Directory {
    pub const fn empty() -> Self {
        Self {
            name: [0; MAX_FILENAME_LEN],
            name_len: 0,
            is_used: false,
            parent_dir: 0,
        }
    }
}

pub struct FileSystem {
    files: [File; MAX_FILES],
    directories: [Directory; MAX_DIRS],
    current_dir: usize,
}

impl FileSystem {
    pub const fn new() -> Self {
        let mut fs = Self {
            files: [File::empty(); MAX_FILES],
            directories: [Directory::empty(); MAX_DIRS],
            current_dir: 0,
        };
        
        fs.directories[0].is_used = true;
        fs.directories[0].name[0] = b'/';
        fs.directories[0].name_len = 1;
        fs.directories[0].parent_dir = 0;
        
        fs
    }

    pub fn init(&mut self) {
        let welcome_content = b"Welcome to HyzeOS!\n\nThis is a simple in-memory file system.\nUse 'help' to see available commands.\n";
        let _ = self.create_file(b"readme.hx", welcome_content);
    }

    pub fn create_file(&mut self, name: &[u8], content: &[u8]) -> Result<(), &'static str> {
        if name.len() > MAX_FILENAME_LEN {
            return Err("Filename too long");
        }
        if content.len() > MAX_FILE_SIZE {
            return Err("File content too large");
        }

        for file in self.files.iter() {
            if file.is_used && file.parent_dir == self.current_dir 
                && file.name_len == name.len() 
                && &file.name[..file.name_len] == name 
            {
                return Err("File already exists");
            }
        }

        for file in self.files.iter_mut() {
            if !file.is_used {
                file.name[..name.len()].copy_from_slice(name);
                file.name_len = name.len();
                file.data[..content.len()].copy_from_slice(content);
                file.size = content.len();
                file.is_used = true;
                file.parent_dir = self.current_dir;
                return Ok(());
            }
        }

        Err("No space for new file")
    }

    pub fn read_file(&self, name: &[u8]) -> Option<&[u8]> {
        for file in self.files.iter() {
            if file.is_used && file.parent_dir == self.current_dir
                && file.name_len == name.len() 
                && &file.name[..file.name_len] == name 
            {
                return Some(&file.data[..file.size]);
            }
        }
        None
    }

    pub fn write_file(&mut self, name: &[u8], content: &[u8]) -> Result<(), &'static str> {
        if content.len() > MAX_FILE_SIZE {
            return Err("Content too large");
        }

        for file in self.files.iter_mut() {
            if file.is_used && file.parent_dir == self.current_dir
                && file.name_len == name.len() 
                && &file.name[..file.name_len] == name 
            {
                file.data[..content.len()].copy_from_slice(content);
                file.size = content.len();
                return Ok(());
            }
        }

        self.create_file(name, content)
    }

    pub fn append_file(&mut self, name: &[u8], content: &[u8]) -> Result<(), &'static str> {
        for file in self.files.iter_mut() {
            if file.is_used && file.parent_dir == self.current_dir
                && file.name_len == name.len() 
                && &file.name[..file.name_len] == name 
            {
                if file.size + content.len() > MAX_FILE_SIZE {
                    return Err("File would exceed max size");
                }
                file.data[file.size..file.size + content.len()].copy_from_slice(content);
                file.size += content.len();
                return Ok(());
            }
        }

        Err("File not found")
    }

    pub fn delete_file(&mut self, name: &[u8]) -> Result<(), &'static str> {
        for file in self.files.iter_mut() {
            if file.is_used && file.parent_dir == self.current_dir
                && file.name_len == name.len() 
                && &file.name[..file.name_len] == name 
            {
                file.is_used = false;
                file.size = 0;
                file.name_len = 0;
                return Ok(());
            }
        }
        Err("File not found")
    }

    pub fn list_files(&self) -> FileIterator {
        FileIterator {
            files: &self.files,
            directories: &self.directories,
            current_dir: self.current_dir,
            file_index: 0,
            dir_index: 0,
            phase: ListPhase::Directories,
        }
    }

    pub fn file_exists(&self, name: &[u8]) -> bool {
        for file in self.files.iter() {
            if file.is_used && file.parent_dir == self.current_dir
                && file.name_len == name.len() 
                && &file.name[..file.name_len] == name 
            {
                return true;
            }
        }
        false
    }

    pub fn create_directory(&mut self, name: &[u8]) -> Result<(), &'static str> {
        if name.len() > MAX_FILENAME_LEN {
            return Err("Directory name too long");
        }

        for dir in self.directories.iter() {
            if dir.is_used && dir.parent_dir == self.current_dir
                && dir.name_len == name.len()
                && &dir.name[..dir.name_len] == name
            {
                return Err("Directory already exists");
            }
        }

        for i in 1..MAX_DIRS {
            if !self.directories[i].is_used {
                self.directories[i].name[..name.len()].copy_from_slice(name);
                self.directories[i].name_len = name.len();
                self.directories[i].is_used = true;
                self.directories[i].parent_dir = self.current_dir;
                return Ok(());
            }
        }

        Err("No space for new directory")
    }

    pub fn change_directory(&mut self, name: &[u8]) -> Result<(), &'static str> {
        if name == b".." {
            if self.current_dir == 0 {
                return Ok(()); // Already at root
            }
            self.current_dir = self.directories[self.current_dir].parent_dir;
            return Ok(());
        }

        if name == b"/" {
            self.current_dir = 0;
            return Ok(());
        }

        for i in 0..MAX_DIRS {
            if self.directories[i].is_used && self.directories[i].parent_dir == self.current_dir
                && self.directories[i].name_len == name.len()
                && &self.directories[i].name[..self.directories[i].name_len] == name
            {
                self.current_dir = i;
                return Ok(());
            }
        }

        Err("Directory not found")
    }

    pub fn get_current_path(&self, buffer: &mut [u8; MAX_PATH_LEN]) -> usize {
        if self.current_dir == 0 {
            buffer[0] = b'/';
            return 1;
        }

        let mut path_parts: [[u8; MAX_FILENAME_LEN]; 8] = [[0; MAX_FILENAME_LEN]; 8];
        let mut part_lens: [usize; 8] = [0; 8];
        let mut depth = 0;
        let mut current = self.current_dir;

        while current != 0 && depth < 8 {
            let dir = &self.directories[current];
            path_parts[depth][..dir.name_len].copy_from_slice(&dir.name[..dir.name_len]);
            part_lens[depth] = dir.name_len;
            current = dir.parent_dir;
            depth += 1;
        }

        let mut pos = 0;
        for i in (0..depth).rev() {
            buffer[pos] = b'/';
            pos += 1;
            buffer[pos..pos + part_lens[i]].copy_from_slice(&path_parts[i][..part_lens[i]]);
            pos += part_lens[i];
        }

        pos
    }

    pub fn remove_directory(&mut self, name: &[u8]) -> Result<(), &'static str> {
        let mut dir_index = None;
        for i in 1..MAX_DIRS {
            if self.directories[i].is_used && self.directories[i].parent_dir == self.current_dir
                && self.directories[i].name_len == name.len()
                && &self.directories[i].name[..self.directories[i].name_len] == name
            {
                dir_index = Some(i);
                break;
            }
        }

        let idx = dir_index.ok_or("Directory not found")?;

        for file in self.files.iter() {
            if file.is_used && file.parent_dir == idx {
                return Err("Directory not empty");
            }
        }
        for i in 1..MAX_DIRS {
            if self.directories[i].is_used && self.directories[i].parent_dir == idx {
                return Err("Directory not empty");
            }
        }

        self.directories[idx].is_used = false;
        self.directories[idx].name_len = 0;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum ListPhase {
    Directories,
    Files,
    Done,
}

pub struct FileIterator<'a> {
    files: &'a [File; MAX_FILES],
    directories: &'a [Directory; MAX_DIRS],
    current_dir: usize,
    file_index: usize,
    dir_index: usize,
    phase: ListPhase,
}

pub enum FileEntry<'a> {
    File(&'a [u8], usize),
    Directory(&'a [u8]),
}

impl<'a> Iterator for FileIterator<'a> {
    type Item = FileEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.phase {
                ListPhase::Directories => {
                    while self.dir_index < MAX_DIRS {
                        let i = self.dir_index;
                        self.dir_index += 1;
                        
                        let dir = &self.directories[i];
                        if dir.is_used && dir.parent_dir == self.current_dir && i != 0 {
                            return Some(FileEntry::Directory(&dir.name[..dir.name_len]));
                        }
                    }
                    self.phase = ListPhase::Files;
                }
                ListPhase::Files => {
                    while self.file_index < MAX_FILES {
                        let i = self.file_index;
                        self.file_index += 1;
                        
                        let file = &self.files[i];
                        if file.is_used && file.parent_dir == self.current_dir {
                            return Some(FileEntry::File(&file.name[..file.name_len], file.size));
                        }
                    }
                    self.phase = ListPhase::Done;
                }
                ListPhase::Done => return None,
            }
        }
    }
}

static mut FILE_SYSTEM: FileSystem = FileSystem::new();

pub fn get_filesystem() -> &'static mut FileSystem {
    unsafe { &mut FILE_SYSTEM }
}
