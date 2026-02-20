# Slint Kanban

A modern, file-system-based Kanban board application built with Rust and Slint UI framework. Manage your tasks with a beautiful, native desktop interface while keeping all your data in plain text files.

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70+-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## ✨ Features

### 🎯 Core Functionality
- **Visual Kanban Board**: Clean, intuitive column-based task management interface
- **Drag & Drop**: Seamlessly move tickets between queues with mouse interactions
- **Real-time Synchronization**: Automatic board updates when files change on disk
- **File-System Storage**: All data stored as plain text files - you own the data
- **YAML Frontmatter**: Pandoc-compatible metadata in Markdown files
- **Cross-referencing**: Link tickets using short IDs (e.g., `#tick12`) (TODO)

### 📝 Ticket Management
- **Create**: Quick ticket creation with title and description
- **Read**: View ticket details with full Markdown rendering
- **Update**: Edit ticket content with live preview
- **Delete**: Safe deletion (moves to `~/Kanban/Deleted` directory)
- **Timestamps**: Automatic creation and modification date tracking
- **Customizable queues**: Queues are just directories in Queue directory.

### 🎨 User Interface
- **Native Performance**: Built with Slint for smooth, responsive UI
- **Ticket Cards Display**:
  - Ticket title and short ID
  - Creation and last modification dates
  - First line snippet of description
  - Overflow protection with ellipsis
- **Queue Management**: Alphabetically sorted queues with customizable names

## 🏗️ Architecture

### File System Structure

```
~/Kanban/
├── Queue/              # Kanban queues (columns), as many as you want
│   ├── 1. Incoming/    # Symlinks to tickets
│   ├── 2. ToDo/
│   ├── 3. Doing/
│   ├── 4. Done/
│   └── 5. Archive/
├── Tickets/            # Actual ticket storage
│   ├── abc123/         # Ticket directory (short ID)
│   │   └── README.md   # Ticket content with YAML frontmatter
│   └── def456/
│       └── README.md
├── Deleted/            # Soft-deleted tickets
└── config.toml         # Shared board settings (users, WIP limits)
~/.config/slint-kanban/
└── user.toml           # Local user settings (active user, search history, hidden queues)
```

### Ticket Format

Each ticket is stored as a directory containing a `README.md` file with YAML frontmatter:

```markdown
---
title: Implement user authentication
created_at: 2026-02-17 10:30:00
updated_at: 2026-02-17 12:15:00
---

## 🚀 Installation

### Prerequisites

- **Rust**: 1.70 or later
- **Linux**: Optimized for Alma Linux 10 (works on other distributions)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/slint_kanban.git
cd slint_kanban

# Build the project
cargo install --path=.

# Run the application using ~/Kanban directory
slint_kanban
```

### Using Pre-built Releases

Download the latest release from the [Releases](https://github.com/yourusername/slint_kanban/releases) page.

## 📖 Usage

### Basic Usage

```bash
# Use default location (~/Kanban)
slint_kanban

# Specify custom Kanban directory
slint_kanban /path/to/your/kanban
```

### First Run

On first launch, the application automatically creates:
- Default directory structure
- Five standard queues: Incoming, ToDo, Doing, Done, Archive

### Working with Tickets

1. **Create a Ticket**: Click the "+" button in any queue
2. **View Details**: Click on a ticket card to see full content
3. **Edit**: Click the "Edit" button in the detail view
4. **Move**: Drag and drop tickets between queues
5. **Delete**: Click the delete button (moves to `~/Kanban/Deleted`)

### External Editing

You can edit tickets using any text editor:

```bash
# Edit a ticket
vim ~/Kanban/Tickets/T-abc123/README.md

# The UI will automatically refresh when you save
```

### Custom Queues

Create custom queues by adding directories:

```bash
mkdir ~/Kanban/Queue/"6. Review"
mkdir ~/Kanban/Queue/"7. Testing"
```

The application will automatically detect and display new queues.

## 🛠️ Technology Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust 2021 Edition |
| UI Framework | [Slint](https://slint.dev/) 1.15 |
| Serialization | serde + serde_yaml |
| File Watching | notify 8.2 |
| Date/Time | chrono 0.4 |
| Error Handling | anyhow 1.0 |

## 🎯 Roadmap

- [ ] Markdown rendering in ticket view
- [ ] Queue limit on tickets per queue setting and enforcement
- [ ] Ticket cross-reference navigation
- [ ] Search and filter functionality
- [ ] Keyboard shortcuts
- [ ] Theme customization
- [ ] Export to various formats (PDF, HTML)
- [ ] Statistics and analytics dashboard

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🛠️ Development & Testing
 
To ensure application stability, run the full test suite with:
 
```bash
cargo test -- --test-threads=1 --nocapture
```
 
Always run `cargo fmt` before finishing a task.
 
## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with [Slint UI](https://slint.dev/) - a declarative GUI toolkit
- Inspired by Trello and other Kanban board applications
- Thanks to the Rust community for excellent tooling and libraries

## 📞 Support

For questions, issues, or feature requests, please [open an issue](https://github.com/yourusername/slint_kanban/issues) on GitHub.

---

**Made with ❤️ using Rust and Slint**
