import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { addLabel, removeLabel } from '../actions';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import AddLabel from './AddLabel.js';
import './LabelsEditor.css';

class LabelsEditor extends Component {
  static propTypes = {
    labels: PropTypes.array.isRequired,
    addLabel: PropTypes.func.isRequired,
    removeLabel: PropTypes.func.isRequired,
    isOpened: PropTypes.bool.isRequired
  }

  render() {
    const {addLabel, removeLabel, labels} = this.props;
    return (
      <div className={`labels-editor${ !this.props.isOpened ? ' hidden' : ''}`}>
        <h1>Labels Editor</h1>
        <div className="editor-labels">
          {labels.map(l => {
            const onRemoveLabelClick = () => removeLabel(l.name);
            return (
              <div key={l.name} className="editor-label-wrapper">
                <div className="label"
                  style={{backgroundColor: l.color}}
                >
                  {l.name}
                </div>
                <a className="label-delete" onClick={onRemoveLabelClick}>
                  <span role="img" aria-label="Delete">✖️</span>
                </a>
              </div>
            );
          })}
        </div>
        <AddLabel addLabel={addLabel} />
      </div>
    );
  }
}

const mapStateToProps = (state) => ({
  labels: state.labels
});

const mapDispatchToProps = (dispatch) => ({
  ...bindActionCreators({ addLabel, removeLabel }, dispatch)
});

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(LabelsEditor);
