import React, { Component } from 'react';
import PropTypes from 'prop-types';
import './DatesChooser.css';

// Convert a date object to a yyyy-mm-dd string.
function convertToYYYYMMDD(date) {
  if (!date) {
    return '';
  }
  return new Date(date).toISOString().substr(0, 10);
}

// Inverse operation
function convertToDate(dateStr) {
  if (!dateStr) {
    return null;
  }
  return new Date(dateStr).getTime();
}

class DatesChooser extends Component {

  static propTypes = {
    completionDate: PropTypes.number,
    onTodoChangeCompletionDate: PropTypes.func.isRequired,
  }

  constructor(props) {
    super(props);
    this.state = { newCompletionDate: convertToYYYYMMDD(this.props.completionDate) };
  }

  handleCompletionDateChange = (e) => {
    const newCompletionDate = e.target.value;
    if (newCompletionDate !== '' && new Date(newCompletionDate) > new Date()) {
      return;
    }
    this.setState({ newCompletionDate });
    this.props.onTodoChangeCompletionDate(convertToDate(newCompletionDate));
  }

  onSubmit = (e) => {
    e.preventDefault();
  }

  render() {
    return (
      <div className="todo-dates-chooser">
        <span>Completion Date</span>
        <input type="date" value={this.state.newCompletionDate}
          onChange={this.handleCompletionDateChange} />
      </div>
    );
  }
}

export default DatesChooser;
