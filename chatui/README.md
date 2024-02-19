## Init

WebStudio: <span style="background-color: #9035c8">Share</span> -> Create Link -> ... -> Build -> <span style="background-color: #096cfe">Copy Link</span>
Terminal:

1. `npx webstudio` -> `Y` -> _`folder_name`_ -> Paste Link -> `Vanilla` -> `Y`
1. `echo "/node_modules\n/build\n/public/build\n.cache\n"` > _`folder_name`_`/.gitignore`
1. `cd `_`folder_name`_

## UI Edits

Any edits made from within webstudio must be published to WebStudio cloud before NPX Webstudio will pick them up

### Procedure

1. webstudio: <span style="background-color: #007a42">Publish</span> -> <span style="background-color: #007a42">Publish</span>
2. terminal: `npx webstudio`
