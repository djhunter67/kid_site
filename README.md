# Purpose
Use my skills as a web developer to build a web app that ingests will keep track of extraneous data throughout my children's childhood.\
There are two children, Adrian and Corbin. I will be keeping track of dental and doctor visits, school events, and pictures.\
This web application is designed to persist data throughout the children of my children's adulthood.  Inevitablely, The site will grow to add\
other family members' children and or adults.  There will be faculties available to add new data corridors in the future.

## Features
- [ ] Plan the project
  - [X] Architecture
  - [ ] Design
  - [ ] Implementation
  - [ ] Testing
- [X] Design the web app to live online
- [ ] Implement the design using HTML and SCSS
- [ ] Ingest data from the logged in user
- [ ] Make the data retrievable by a logged in user
- [ ] Allow the user to add another person
- [ ] Track user actions over time
- [ ] User login and authentication
  - [X] Encrypt, salt, and store pw
  - [ ] Reset password
  - [ ] Email confirmation
- [ ] Export a report of user activity over time in PDF format
- [X] Free Free Free
- [ ] Roll my own authorization
- [X] Document database wired up to the app
- [X] Locally host to the world ([web_app](education.hunter-homelab.com))

## Architecture
To keep any users with minimal computer or web application experience from being overwhelmed the bones of the application will be simple and to the point. The application will be backend heavy.  The pattern of use will be mirrored for every subject for which data is submitted.

## Design
There will be an aim to keep buttons, cards, and shapes uniform and aligned throughout the user experience. Text that will be a link will be uppercase and no boxes will have sharp corners.  Colors will be soft and text will be high contrast to its background. 

### User Experience
Attention to few on screen distractions will be paid.  There are currently no plans to gamify, thus there will be no effort to implement frills, poppers, etc... 

## Design Ideology
Optimizing for future expansion and a simplistic is the primary concern while developing due to the fact that the intended user group are non-technical family members.  Inherent in the design age group is the number of intended users is one but design will support multiple users.  Thus, simple expansion to different, more, or complex subjects will be attainable with minimal time and effort. This is a golden opportunity to roll a custom auth to know if that is something to be avoided in the future.

## Technologies
### Frontend
- HTML
- SCSS
- HTMX
- Javascript

The technologies used in the frontend are the __best tools for the job__.  To be clear, the __best tool for the job__ is compeletely subjective.  The __best tool for the job__ is the tool that is most familiar to the developer.  It is believed that familiarization with tools upon which most other tools are built is critical to a well-rounded developer.  Thus, the __best tools for the job__ are HTML, SCSS, and HTMX.  Experience with Bulma, Tailwind, and Bootstrap are strongly opinionated libraries and, admittedly, would work well in this scenario but there is more to learn abut CSS and every project is an opportunity to grow.

### Backend
- Rust
- Actix-Web
- MongoDB
- Linux (Arch, BTW, for development)

Docuemnt databases are inately different from relational databases.  Learning about the key differences in practice versus reading about the differences will allow for a more well-rounded developer.  Rust is a language that is not only fun to write but is also a language that is type safe and fast. 

