---
name: Pull Request
description: Create a pull request
title: "[PR] "
labels: []
assignees: ["j4v3l"]
body:
  - type: markdown
    attributes:
      value: |
        ## Pull Request

        Thanks for contributing! Please fill out the information below.

  - type: textarea
    id: description
    attributes:
      label: Description
      description: What does this PR do? What problem does it solve?
      placeholder: "This PR adds... to fix..."
    validations:
      required: true

  - type: dropdown
    id: type
    attributes:
      label: Type of Change
      description: What type of change does this PR introduce?
      options:
        - Bug fix
        - New feature
        - Documentation update
        - Code refactoring
        - Performance improvement
        - Security enhancement
        - Other (please specify)
    validations:
      required: true

  - type: textarea
    id: related
    attributes:
      label: Related Issues
      description: Link to related issues (if any)
      placeholder: "Fixes #123, Addresses #456"

  - type: textarea
    id: testing
    attributes:
      label: Testing
      description: How have you tested this change?
      placeholder: |
        - [ ] Unit tests pass
        - [ ] Integration tests pass
        - [ ] Manual testing completed
        - [ ] Cross-platform testing done
    validations:
      required: true

  - type: textarea
    id: checklist
    attributes:
      label: Checklist
      description: Please check all that apply.
      value: |
        - [ ] Code compiles without warnings
        - [ ] All tests pass (`cargo test`)
        - [ ] Code formatted (`cargo fmt`)
        - [ ] Clippy warnings fixed (`cargo clippy`)
        - [ ] Documentation updated
        - [ ] Breaking changes documented
        - [ ] Commit messages are clear
    validations:
      required: true

  - type: textarea
    id: additional
    attributes:
      label: Additional Notes
      description: Any additional information or context?
