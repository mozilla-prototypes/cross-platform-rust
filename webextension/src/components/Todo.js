import React, { Component } from 'react';
import PropTypes from 'prop-types';
import TodoDropdownMenu from './TodoDropdownMenu';
import LabelsChooser from './LabelsChooser';
import DatesChooser from './DatesChooser';
import './Todo.css';

function toPrettyDate(date) {
  return new Date(date).toISOString().substr(0, 10);
}

class Todo extends Component {
  static propTypes = {
    onRemoveTodo: PropTypes.func.isRequired,
    onTodoLabelAdded: PropTypes.func.isRequired,
    onTodoLabelRemoved: PropTypes.func.isRequired,
    onTodoNameChanged: PropTypes.func.isRequired,
    onTodoCompleted: PropTypes.func.isRequired,
    onTodoChangeDueDate: PropTypes.func.isRequired,
    onTodoChangeCompletionDate: PropTypes.func.isRequired,
    allLabels: PropTypes.array.isRequired,
    labels: PropTypes.array.isRequired,
    name: PropTypes.string.isRequired,
    dueDate: PropTypes.number,
    completionDate: PropTypes.number,
    uuid: PropTypes.string.isRequired
  }

  state = {
    datesDropdownOpen: false,
    labelsDropdownOpen: false,
    isEditingName: false,
    newName: ''
  }

  onSubmit = (e) => {
    e.preventDefault();
    this.toggleEdit();
  }

  toggleEdit = () => {
    this.setState({isEditingName: !this.state.isEditingName});
    if (!this.state.isEditingName) {
      this.setState({newName: this.props.name});
    } else {
      this.props.onTodoNameChanged(this.props.uuid, this.state.newName);
    }
  }

  handleCompleted = (e) => {
    this.props.onTodoCompleted(this.props.uuid, e.target.checked);
  }

  handleKeyDown = (e) => {
    // Cancel edit on escape key
    if (e.keyCode === 27) {
      this.setState({isEditingName: !this.state.isEditingName});
    }
  }

  toggleLabelsDropdown = () => {
    this.setState({labelsDropdownOpen: !this.state.labelsDropdownOpen});
  }

  toggleDatesDropdown = () => {
    this.setState({datesDropdownOpen: !this.state.datesDropdownOpen});
  }

  handleNewNameChange = (event) => {
    this.setState({newName: event.target.value});
  }

  render() {
    const { uuid, name, labels, dueDate, completionDate, allLabels,
      onRemoveTodo, onTodoLabelAdded, onTodoLabelRemoved,
      onTodoChangeDueDate, onTodoChangeCompletionDate } = this.props;
    const { labelsDropdownOpen, datesDropdownOpen, isEditingName } = this.state;

    const labelsChecked = new Set(labels.map(l => l.name));
    const availableLabels = allLabels.map(l => {
      return Object.assign({}, l, { checked: labelsChecked.has(l.name)});
    });

    const onLabelChecked = (labelName) => onTodoLabelAdded(uuid, labelName);
    const onLabelUnchecked = (labelName) => onTodoLabelRemoved(uuid, labelName);

    const todoChangeDueDate = (date) => onTodoChangeDueDate(uuid, date);
    const todoChangeCompletionDate = (date) => onTodoChangeCompletionDate(uuid, date);

    return (
      <div className="todo-wrapper">
        <div className={`todo${ labelsDropdownOpen || datesDropdownOpen ? ' dropdown-open' : ''}`}>
          <input className="todo-completed" checked={!!completionDate} onChange={this.handleCompleted} type="checkbox"
            title={completionDate? `Completed on ${ toPrettyDate(completionDate)}` : ''} />
          <div className="todo-content">
            <div className="todo-details-wrapper">
              <div className="todo-details">
                <div className="todo-labels">{labels.map(l => <span key={l.name} style={{backgroundColor: l.color}}>{l.name}</span>)}</div>
                <div className="todo-name">
                  {
                    !isEditingName ?
                      <div onDoubleClick={this.toggleEdit}>{name}</div> :
                      <form onSubmit={this.onSubmit}>
                        <input className="todo-edit-name" onKeyDown={this.handleKeyDown} value={this.state.newName} onChange={this.handleNewNameChange} />
                      </form>
                  }
                </div>
              </div>
              <div className="todo-buttons-wrapper">
                <div className="todo-dropdown-wrapper">
                  <div className="todo-label-button" onClick={this.toggleLabelsDropdown}>
                    <span role="img" aria-label="Label">🏷</span>
                  </div>
                  {labelsDropdownOpen ?
                    <TodoDropdownMenu onCloseDropdown={this.toggleLabelsDropdown}>
                      <LabelsChooser labels={availableLabels} onLabelChecked={onLabelChecked} onLabelUnchecked={onLabelUnchecked} />
                    </TodoDropdownMenu> : null
                  }
                </div>
                <div className="todo-dropdown-wrapper">
                  <div className="todo-dates-button" onClick={this.toggleDatesDropdown}>
                    <span role="img" aria-label="Dates">📅</span>
                  </div>
                  {datesDropdownOpen ?
                    <TodoDropdownMenu onCloseDropdown={this.toggleDatesDropdown}>
                      <DatesChooser dueDate={dueDate} completionDate={completionDate}
                        onTodoChangeDueDate={todoChangeDueDate}
                        onTodoChangeCompletionDate={todoChangeCompletionDate} />
                    </TodoDropdownMenu> : null
                  }
                </div>
                <div className="todo-delete-button" onClick={onRemoveTodo}>
                  <span role="img" aria-label="Delete">❌</span>
                </div>
              </div>
            </div>
            <div className="todo-footer">
              {dueDate && !completionDate ? `Due on ${ toPrettyDate(dueDate)}` : ''}
            </div>
          </div>
        </div>
      </div>
    );
  }
}

export default Todo;
