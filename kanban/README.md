---
title: Project Overview
author: Kanban Authors
created_at: 2026-01-01 00:00:00
updated_at: 2026-01-01 00:00:00
---
# Board Overview

Welcome to your new Kanban board! This system is designed for local-first, file-system-based project management.

## Quality Process (Definition of Done)
To ensure high quality and clarity:
1. **Verification**: A ticket is considered "Done" only when the **original author** confirms that all tasks, conditions, and acceptance criteria have been fully met.
2. **Review**: Tasks should move through the "Reviewing" and "Testing" queues before being finalized.
3. **Closing**: Only after confirmation should a ticket be moved to the "Done" queue. "Archive" is reserved for completed tasks that are no longer needed for daily tracking.

## Statistics & Analytics
Detailed analytics are available via the **Board Info** -> **Statistics** button.

### How Metrics are Calculated:
- **Board Completion Rate**: `(Done Tickets) / (Total Tickets - Archived Tickets)`. This represents the overall progress of the project, excluding tasks that have been archived.
- **Sprint Completion Rate**: This metric tracks performance during the active sprint. It is calculated as `(Completed in Sprint) / (Active in Sprint)`, where "Active" includes any ticket created or modified during the sprint period.
- **Lead Time**: The total time elapsed from the moment a ticket is created until it reaches the "Done" queue. It measures the customer's perspective of time.
- **Cycle Time**: The time spent actively working on a task. It measures the duration from when a ticket leaves the "starting" queues (e.g., To Dooooo) until it enters a "done" queue.

## Sprints
Sprints are time-boxed iterations (usually 1-2 weeks) that help the team focus on a specific set of tasks.
- **Detection**: The system automatically detects the current sprint based on today's date.
- **Tracking**: Use the "Sprint" display in the header to see the current sprint's progress.
- **Management**: You can add, update, or remove sprints using the CLI: `kanban sprint add --name "Sprint Name" --start YYYY-MM-DD --end YYYY-MM-DD`.

## Configuration
Customize your board by editing `config.toml`:
- **users**: Define team members to enable assignment.
- **queue_limits**: Set WIP (Work In Progress) limits to prevent bottlenecks.
- **workflows**: Customize which queues are considered "start" (e.g., To Dooooooo) and "done" (e.g., Done, Archive) for accurate time tracking.
- **points_scale**: Customize point values and their meaning (default setup is 1-10).

## Estimation (Points)
Each ticket can be assigned a "Point" value (from 0 to 10) to represent the estimated effort or complexity:
- **0 pts**: No estimation or trivial task.
- **1-4 pts**: Tasks taking 1 to 4 days.
- **5 pts**: 1 week.
- **6 pts**: 2 weeks.
- **7 pts**: 1 month.
- **8 pts**: 2-3 months.
- **9 pts**: 6 months.
- **10 pts**: 1 year.

The system uses these points to calculate the **Board Completion Rate (by points)**, which provides a more accurate view of progress than just ticket count.

Tip: The `config.toml` file uses the TOML format. The application will automatically reload when you save changes to this file.
